//! Agent commands

pub mod lock;
pub mod start;
pub mod status;
pub mod stop;
pub mod unlock;

use std::path::PathBuf;

use solana_keyring::default_agent_socket_path;

/// Get the socket path, using provided or default
pub fn get_socket_path(path: &Option<PathBuf>) -> PathBuf {
    path.clone().unwrap_or_else(default_agent_socket_path)
}
