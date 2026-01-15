//! Ledger USB/HID transport

use crate::error::{Error, Result};

// Solana app APDU constants
const SOLANA_CLA: u8 = 0xE0;
const INS_GET_PUBKEY: u8 = 0x05;
const INS_SIGN_MESSAGE: u8 = 0x06;

/// Get the public key from the Ledger device
pub fn get_pubkey(derivation_path: &[u32]) -> Result<[u8; 32]> {
    let transport = open_device()?;
    let data = serialize_derivation_path(derivation_path);

    let response = exchange_apdu(&transport, SOLANA_CLA, INS_GET_PUBKEY, 0x00, 0x00, &data)?;

    if response.len() < 32 {
        return Err(Error::Ledger("Invalid public key response".into()));
    }

    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&response[..32]);
    Ok(pubkey)
}

/// Sign a message using the Ledger device
pub fn sign_message(derivation_path: &[u32], message: &[u8]) -> Result<[u8; 64]> {
    let transport = open_device()?;

    let mut data = serialize_derivation_path(derivation_path);
    data.extend_from_slice(message);

    // Chunk data if needed (Ledger has max payload size)
    let chunks: Vec<&[u8]> = data.chunks(255).collect();
    let mut signature = None;

    for (i, chunk) in chunks.iter().enumerate() {
        let p1 = if i == 0 { 0x00 } else { 0x80 };
        let p2 = if i == chunks.len() - 1 { 0x00 } else { 0x80 };

        let response = exchange_apdu(&transport, SOLANA_CLA, INS_SIGN_MESSAGE, p1, p2, chunk)?;

        if i == chunks.len() - 1 {
            signature = Some(response);
        }
    }

    let sig_bytes = signature.ok_or_else(|| Error::Ledger("No signature returned".into()))?;

    if sig_bytes.len() < 64 {
        return Err(Error::Ledger("Invalid signature response".into()));
    }

    let mut sig = [0u8; 64];
    sig.copy_from_slice(&sig_bytes[..64]);
    Ok(sig)
}

/// Open the Ledger device
fn open_device() -> Result<hidapi::HidDevice> {
    let api = hidapi::HidApi::new().map_err(|e| Error::Ledger(e.to_string()))?;

    // Ledger vendor ID
    const LEDGER_VID: u16 = 0x2c97;

    for device in api.device_list() {
        if device.vendor_id() == LEDGER_VID
            && let Ok(dev) = api.open_path(device.path())
        {
            return Ok(dev);
        }
    }

    Err(Error::LedgerNotConnected)
}

/// Serialize derivation path for APDU
fn serialize_derivation_path(path: &[u32]) -> Vec<u8> {
    let mut data = vec![path.len() as u8];
    for &component in path {
        data.extend_from_slice(&component.to_be_bytes());
    }
    data
}

/// Exchange an APDU with the device
fn exchange_apdu(
    device: &hidapi::HidDevice,
    cla: u8,
    ins: u8,
    p1: u8,
    p2: u8,
    data: &[u8],
) -> Result<Vec<u8>> {
    // Build APDU
    let mut apdu = vec![cla, ins, p1, p2, data.len() as u8];
    apdu.extend_from_slice(data);

    // Wrap in HID frame
    let mut frame = vec![0x00]; // Report ID
    frame.push(0x01); // Channel high
    frame.push(0x01); // Channel low
    frame.push(0x05); // Tag
    frame.push(0x00); // Sequence high
    frame.push(0x00); // Sequence low
    frame.push((apdu.len() >> 8) as u8);
    frame.push((apdu.len() & 0xff) as u8);
    frame.extend_from_slice(&apdu);

    // Pad to 65 bytes
    frame.resize(65, 0);

    device
        .write(&frame)
        .map_err(|e| Error::Ledger(e.to_string()))?;

    // Read response
    let mut response = vec![0u8; 65];
    device
        .read_timeout(&mut response, 30000)
        .map_err(|e| Error::Ledger(e.to_string()))?;

    // Parse response (skip HID framing)
    if response.len() < 9 {
        return Err(Error::Ledger("Invalid response".into()));
    }

    let data_len = ((response[5] as usize) << 8) | (response[6] as usize);
    if data_len < 2 {
        return Err(Error::Ledger("Invalid response length".into()));
    }

    // Check status word
    let data_end = 7 + data_len - 2;
    let sw = ((response[data_end] as u16) << 8) | (response[data_end + 1] as u16);

    if sw != 0x9000 {
        return Err(Error::Ledger(format!("Ledger error: 0x{:04X}", sw)));
    }

    Ok(response[7..data_end].to_vec())
}
