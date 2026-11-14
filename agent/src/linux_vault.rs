use rand::RngCore;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};
use std::{io::Write, process::{Command, Stdio}, sync::Mutex};
use zeroize::Zeroizing;

static KEY_ACCESS: Mutex<()> = Mutex::new(());
const ATTRIBUTES: [&str; 2] = ["application", "dev.digitaldiscipline.desktop"];
const ERROR: &str = "linux_secret_service_unavailable";

fn key(create: bool) -> Result<LessSafeKey, String> {
    let _guard = KEY_ACCESS.lock().map_err(|_| ERROR)?;
    let result = Command::new("timeout").args(["10s", "secret-tool", "lookup"])
        .args(ATTRIBUTES).output().map_err(|_| ERROR)?;
    let output = Zeroizing::new(result.stdout);
    let bytes = if result.status.success() {
        let encoded = std::str::from_utf8(&output).map_err(|_| ERROR)?;
        Zeroizing::new(hex::decode(encoded.trim()).map_err(|_| ERROR)?)
    } else if create && result.status.code() == Some(1) && result.stderr.is_empty() {
        let mut bytes = Zeroizing::new(vec![0; 32]);
        rand::thread_rng().fill_bytes(&mut bytes);
        let mut child = Command::new("timeout").args(["10s", "secret-tool", "store", "--label=Digital Discipline vault"])
            .args(ATTRIBUTES).stdin(Stdio::piped()).stdout(Stdio::null()).spawn().map_err(|_| ERROR)?;
        let encoded = Zeroizing::new(hex::encode(&*bytes));
        let wrote = child.stdin.take().ok_or(ERROR)?.write_all(encoded.as_bytes());
        let status = child.wait().map_err(|_| ERROR)?;
        if wrote.is_err() || !status.success() {
            return Err(ERROR.into());
        }
        bytes
    } else {
        return Err(ERROR.into());
    };
    UnboundKey::new(&CHACHA20_POLY1305, &bytes).map(LessSafeKey::new).map_err(|_| ERROR.into())
}

pub fn protect(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut nonce = [0; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    let mut data = Zeroizing::new(bytes.to_vec());
    let tag = key(true)?.seal_in_place_separate_tag(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut data)
        .map_err(|_| "credential_encryption_failed")?;
    let mut result = nonce.to_vec();
    result.extend_from_slice(&data);
    result.extend_from_slice(tag.as_ref());
    Ok(result)
}

pub fn unprotect(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let (nonce, ciphertext) = bytes.split_at_checked(12).ok_or("invalid_vault_blob")?;
    let mut data = Zeroizing::new(ciphertext.to_vec());
    let nonce = Nonce::try_assume_unique_for_key(nonce).map_err(|_| "invalid_vault_blob")?;
    let plaintext = key(false)?.open_in_place(nonce, Aad::empty(), &mut data)
        .map_err(|_| "credential_decryption_failed")?;
    Ok(plaintext.to_vec())
}
