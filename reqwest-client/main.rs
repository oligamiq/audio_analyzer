use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use lib::Msg;
use reqwest_client::run_code;

fn main() {
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
