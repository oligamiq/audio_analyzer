use serde::{Deserialize, Serialize};


#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Code {
    pub code: Option<String>,
}

#[no_mangle]
pub fn run(code: Code) -> String {
    // code.code.unwrap_or_default()
    "Hello, Worlmkd!".to_string()
}
