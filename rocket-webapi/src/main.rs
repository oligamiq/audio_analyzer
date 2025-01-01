#[macro_use]
extern crate rocket;
use rocket::response::content::RawJson;
use rocket::serde::json::Json;
use rocket::Config;

#[hot_lib_reloader::hot_module(dylib = "lib", lib_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/debug")
)]
mod hot_lib {
    // hot_functions_from_file!("rocket-webapi/lib/src/lib.rs");

    #[hot_function]
    pub fn run(code: lib::Code) -> String {
        code.code.unwrap_or_default()
    }

    pub use lib::Code;
}

// curl -X POST -H "Content-Type: application/json" -d {\"code\":\"print\"} http://localhost:1080/
#[post("/", format = "json", data = "<code>")]
fn index(
    code: Json<lib::Code>,
) -> RawJson<String> {
    RawJson(hot_lib::run(code.0).into())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(Config {
            port: 1080,
            address: std::net::Ipv4Addr::new(0, 0, 0, 0).into(),
            log_level: rocket::log::LogLevel::Debug,
            ..Default::default()
        })
        .mount("/", routes![index])
}
