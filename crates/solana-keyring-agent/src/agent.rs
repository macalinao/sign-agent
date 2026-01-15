//! Agent implementation

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine as _;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use zeroize::Zeroizing;

use solana_keyring::{Database, default_db_path, list_signers};

use crate::protocol::{AgentStatus, ErrorCode, Request, Response, ResponseResult, SignerInfo};

/// Agent state
pub struct AgentState {
    pub passphrase: Option<Zeroizing<Vec<u8>>>,
    pub db_path: PathBuf,
    pub unlocked_at: Option<Instant>,
    pub started_at: Instant,
    pub lock_timeout: Duration,
}

impl AgentState {
    pub fn new(db_path: Option<PathBuf>, lock_timeout: Duration) -> Self {
        Self {
            passphrase: None,
            db_path: db_path.unwrap_or_else(default_db_path),
            unlocked_at: None,
            started_at: Instant::now(),
            lock_timeout,
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.passphrase.is_some()
    }

    pub fn unlock(&mut self, passphrase: Vec<u8>) {
        self.passphrase = Some(Zeroizing::new(passphrase));
        self.unlocked_at = Some(Instant::now());
    }

    pub fn lock(&mut self) {
        self.passphrase = None;
        self.unlocked_at = None;
    }

    pub fn check_timeout(&mut self) {
        if let Some(unlocked_at) = self.unlocked_at
            && unlocked_at.elapsed() > self.lock_timeout
        {
            self.lock();
        }
    }
}

/// Agent server
pub struct Agent {
    state: Arc<RwLock<AgentState>>,
    socket_path: PathBuf,
}

impl Agent {
    pub fn new(socket_path: PathBuf, db_path: Option<PathBuf>, lock_timeout: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(AgentState::new(db_path, lock_timeout))),
            socket_path,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        // Remove existing socket file
        let _ = std::fs::remove_file(&self.socket_path);

        // Create parent directory
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;

        // Set socket permissions (owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.socket_path, std::fs::Permissions::from_mode(0o600))?;
        }

        println!("Agent listening on {}", self.socket_path.display());

        // Spawn timeout checker
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let mut state = state_clone.write().await;
                state.check_timeout();
            }
        });

        // Accept connections
        loop {
            let (stream, _) = listener.accept().await?;
            let state = self.state.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, state).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }
}

async fn handle_connection(
    mut stream: UnixStream,
    state: Arc<RwLock<AgentState>>,
) -> anyhow::Result<()> {
    let mut len_buf = [0u8; 4];

    loop {
        // Read length prefix
        match stream.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Client disconnected
                break;
            }
            Err(e) => return Err(e.into()),
        }

        let len = u32::from_be_bytes(len_buf) as usize;
        if len == 0 || len > 1_048_576 {
            break;
        }

        // Read message
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;

        // Parse and process request
        let response = match serde_json::from_slice::<Request>(&buf) {
            Ok(request) => process_request(request, &state).await,
            Err(e) => Response::error(ErrorCode::InternalError, e.to_string()),
        };

        // Send response
        let response_bytes = serde_json::to_vec(&response)?;
        stream
            .write_all(&(response_bytes.len() as u32).to_be_bytes())
            .await?;
        stream.write_all(&response_bytes).await?;

        // Check for shutdown request
        if matches!(
            serde_json::from_slice::<Request>(&buf),
            Ok(Request::Shutdown)
        ) {
            std::process::exit(0);
        }
    }

    Ok(())
}

async fn process_request(request: Request, state: &Arc<RwLock<AgentState>>) -> Response {
    match request {
        Request::Ping => Response::ok(ResponseResult::Pong),

        Request::Status => {
            let state = state.read().await;
            Response::ok(ResponseResult::Status(AgentStatus {
                unlocked: state.is_unlocked(),
                uptime_seconds: state.started_at.elapsed().as_secs(),
                signer_count: 0, // TODO: count signers
                lock_timeout_seconds: state.lock_timeout.as_secs(),
            }))
        }

        Request::Unlock { passphrase } => {
            let mut state = state.write().await;

            // Verify passphrase
            let db = match Database::open(&state.db_path) {
                Ok(db) => db,
                Err(e) => return Response::error(ErrorCode::InternalError, e.to_string()),
            };

            match db.verify_passphrase(passphrase.as_bytes()) {
                Ok(true) => {
                    state.unlock(passphrase.into_bytes());
                    Response::ok(ResponseResult::Unit)
                }
                Ok(false) => Response::error(ErrorCode::InvalidPassphrase, "Invalid passphrase"),
                Err(e) => Response::error(ErrorCode::InternalError, e.to_string()),
            }
        }

        Request::Lock => {
            let mut state = state.write().await;
            state.lock();
            Response::ok(ResponseResult::Unit)
        }

        Request::ListSigners { tag } => {
            let state = state.read().await;

            let db = match Database::open(&state.db_path) {
                Ok(db) => db,
                Err(e) => return Response::error(ErrorCode::InternalError, e.to_string()),
            };

            match list_signers(&db, tag.as_deref()) {
                Ok(signers) => {
                    let infos: Vec<SignerInfo> = signers
                        .into_iter()
                        .map(|s| SignerInfo {
                            pubkey: s.pubkey,
                            label: s.label,
                            signer_type: s.signer_type.to_string(),
                            tags: s.tags,
                        })
                        .collect();
                    Response::ok(ResponseResult::Signers(infos))
                }
                Err(e) => Response::error(ErrorCode::InternalError, e.to_string()),
            }
        }

        Request::SignTransaction {
            transaction,
            signer,
        } => {
            let state = state.read().await;

            if !state.is_unlocked() {
                return Response::error(ErrorCode::Locked, "Agent is locked");
            }

            let passphrase = state.passphrase.as_ref().unwrap();

            let db = match Database::open(&state.db_path) {
                Ok(db) => db,
                Err(e) => return Response::error(ErrorCode::InternalError, e.to_string()),
            };

            // Decode transaction
            let tx_bytes: Vec<u8> =
                match base64::engine::general_purpose::STANDARD.decode(&transaction) {
                    Ok(b) => b,
                    Err(e) => return Response::error(ErrorCode::InvalidTransaction, e.to_string()),
                };

            // Parse transaction to show details to user
            let summary = match solana_keyring::transaction::summarize_transaction(&tx_bytes) {
                Ok(s) => s.to_string(),
                Err(_) => "Unable to parse transaction details".to_string(),
            };

            // Get signer label for display
            let signer_label = db
                .list_keypairs(None)
                .ok()
                .and_then(|keypairs| {
                    keypairs
                        .into_iter()
                        .find(|k| k.pubkey == signer || k.label == signer)
                        .map(|k| k.label)
                })
                .unwrap_or_else(|| signer.clone());

            // Request biometric/user confirmation
            use solana_keyring::biometric::AuthResult;
            match solana_keyring::biometric::confirm_signing(&signer_label, &summary) {
                Ok(AuthResult::Authenticated) => {
                    // User confirmed, proceed with signing
                }
                Ok(AuthResult::Denied) => {
                    return Response::error(ErrorCode::InternalError, "User cancelled signing");
                }
                Ok(AuthResult::NotAvailable) => {
                    // Biometrics not available, proceed without confirmation
                    eprintln!(
                        "Biometric authentication not available, proceeding without confirmation"
                    );
                }
                Err(e) => {
                    // If biometric fails, log but continue (non-fatal)
                    eprintln!("Biometric check failed: {}", e);
                }
            }

            // Load keypair and sign
            match db.load_keypair(&signer, passphrase) {
                Ok(keypair) => {
                    let signature = keypair.sign(&tx_bytes);
                    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(signature);

                    // Send notification
                    let _ = solana_keyring::notify(
                        "Transaction Signed",
                        &format!("Signed with {}", signer_label),
                    );

                    Response::ok(ResponseResult::SignedTransaction(sig_b64))
                }
                Err(e) => Response::error(ErrorCode::SignerNotFound, e.to_string()),
            }
        }

        Request::Shutdown => {
            // Handled after response is sent
            Response::ok(ResponseResult::Unit)
        }
    }
}
