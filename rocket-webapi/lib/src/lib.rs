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
    println!("rewriting code");

    rewrite_code(code.clone());

    let file_pos_base = std::concat!(std::env!("CARGO_MANIFEST_DIR"), "/../..");

    // let out = std::process::Command::new("cargo")
    //     .arg("run")
    //     .arg("--release")
    //     .arg("--package")
    //     .arg("audio_analyzer_inner_checker")
    //     .current_dir(file_pos_base)
    //     .output()
    //     .expect("failed to execute process");

    Msg::CompileStart
    // Msg::CompileEnd
}

pub fn rewrite_code(code: Code) {
    let Code { code } = code;

    let file_pos_base = std::concat!(std::env!("CARGO_MANIFEST_DIR"), "/../..");

    let file_pos = format!(r"{file_pos_base}/audio_analyzer_inner_checker/src/fn_.rs");

    println!("file_pos: {}", file_pos);

    // overwrite file
    std::fs::write(file_pos, code).unwrap();

    // fmt
    let out = std::process::Command::new("cargo")
        .arg("fmt")
        .current_dir(file_pos_base)
        .output()
        .expect("failed to execute process");

    println!("out: {}", String::from_utf8_lossy(&out.stdout));
    println!("err: {}", String::from_utf8_lossy(&out.stderr));
}
