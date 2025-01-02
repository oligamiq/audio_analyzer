use serde::{Deserialize, Serialize};


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Code {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Msg {
    CompileStart,
    CompileEnd,
}

#[no_mangle]
pub fn run(code: String) -> String {
    serde_json::to_string(&run_inner(serde_json::from_str(&code).unwrap())).unwrap()
}

pub fn run_inner(code: Code) -> Msg {
    Msg::CompileStart
    // Msg::CompileEnd
}
