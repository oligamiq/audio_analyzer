use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use lib::Msg;

pub use lib;

#[cfg(target_arch = "wasm32")]
pub async fn get(code: String) -> anyhow::Result<lib::Msg> {
    use reqwest::header::CONTENT_TYPE;
    use reqwest::Client;

    // Create a reqwest Client with rustls
    let client = Client::builder().build()?;

    // Define the URL and payload
    let url = "http://localhost:1080/";
    let payload = lib::Code { code: code };

    println!("Sending code: {}", payload.code);

    // Send the POST request
    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&payload)?)
        .send()
        .await?;

    let msg_str = response.text().await?;
    let msg = serde_json::from_str(&msg_str)?;

    Ok(msg)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_blocking(code: String) -> anyhow::Result<lib::Msg> {
    // Define the URL and payload
    let url = "http://localhost:1080/";
    let payload = lib::Code { code: code };

    println!("Sending code: {}", payload.code);

    // Send the POST request
    let response = ureq::post(url)
        .set("Content-Type", "application/json")
        .send_json(serde_json::to_value(&payload)?)?
        .into_string()?;

    let msg = match serde_json::from_str(&response) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("Error: {} (response: {})", e, response);
            return Err(e.into());
        }
    };

    Ok(msg)
}

pub fn run_code(code: String, queue: Arc<Mutex<VecDeque<Msg>>>) -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")]
    async fn run_code_inner(code: String, queue: Arc<Mutex<VecDeque<Msg>>>) -> anyhow::Result<()> {
        let result = get(code).await;
        let mut queue = queue.lock().unwrap();
        queue.push_back(result?);

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn block_run_code(code: String, queue: Arc<Mutex<VecDeque<Msg>>>) -> anyhow::Result<()> {
        let result = get_blocking(code);
        let mut queue = queue.lock().unwrap();
        queue.push_back(result?);

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = run_code_inner(code, queue).await {
            eprintln!("Error: {}", e);
        }
    });

    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        if let Err(e) = block_run_code(code, queue) {
            eprintln!("Error: {}", e);
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_code() {
        println!("Starting test_run_code");

        let code = "fn main() { println!(\"Hello, world!\"); }".to_string();
        let queue = Arc::new(Mutex::new(VecDeque::new()));

        println!("Running code: {}", code);

        assert!(run_code(code, queue.clone()).is_ok());

        loop {
            if let Ok(mut q) = queue.clone().try_lock() {
                if let Some(msg) = q.pop_front() {
                    assert_eq!(msg, Msg::CompileStart);
                    break;
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
