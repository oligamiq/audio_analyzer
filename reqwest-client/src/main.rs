use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;

fn main() -> anyhow::Result<()> {
    // Create a reqwest Client with rustls
    let client = Client::builder()
        .use_rustls_tls()
        .build()?;

    // Define the URL and payload
    let url = "http://localhost:1080/";
    let payload = r#"{"code":"print"}"#;

    // Send the POST request
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .body(payload)
        .send()?;

    // Print the response status and body
    println!("Status: {}", response.status());
    if let Ok(text) = response.text() {
        println!("Response: {}", text);
    }

    Ok(())
}
