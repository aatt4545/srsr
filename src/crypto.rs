// src/crypto.rs
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const EXTENSION: &str = ".encrypted";

fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

fn encrypt_file(path: &Path, key: &[u8; 32]) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, data.as_ref()).map_err(|e| e.to_string())?;
    
    let mut encrypted_data = Vec::new();
    encrypted_data.extend_from_slice(&nonce_bytes);
    encrypted_data.extend_from_slice(&ciphertext);
    
    let new_path = format!("{}{}", path.display(), EXTENSION);
    fs::write(&new_path, &encrypted_data).map_err(|e| e.to_string())?;
    fs::remove_file(path).map_err(|e| e.to_string())?;
    
    Ok(())
}

fn decrypt_file(path: &Path, key: &[u8; 32]) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    
    if data.len() < 12 {
        return Err("Invalid encrypted file".to_string());
    }
    
    let nonce_bytes: [u8; 12] = data[0..12].try_into().map_err(|_| "Invalid nonce".to_string())?;
    let ciphertext = &data[12..];
    
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())?;
    
    let original_path = path.display().to_string().replace(EXTENSION, "");
    fs::write(&original_path, &plaintext).map_err(|e| e.to_string())?;
    fs::remove_file(path).map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn encrypt_directories(dirs: &[PathBuf], password: &str) {
    let key = derive_key(password);
    
    for dir in dirs {
        for entry in WalkDir::new(dir).follow_links(false) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if !path.extension().map_or(false, |e| e == "encrypted") {
                        let _ = encrypt_file(path, &key);
                    }
                }
            }
        }
    }
}

pub fn decrypt_directories(dirs: &[PathBuf], password: &str) {
    let key = derive_key(password);
    
    for dir in dirs {
        for entry in WalkDir::new(dir).follow_links(false) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "encrypted") {
                        let _ = decrypt_file(path, &key);
                    }
                }
            }
        }
    }
}
