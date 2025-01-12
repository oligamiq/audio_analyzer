#[macro_use]
extern crate rocket;
use rocket::response::{content::RawJson, Responder};
use rocket::serde::json::Json;
use rocket::{Config, Response};

#[hot_lib_reloader::hot_module(dylib = "lib", lib_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/debug")
)]
mod hot_lib {
    // hot_functions_from_file!("rocket-webapi/lib/src/lib.rs");

    #[hot_functions]
    extern "Rust" {
        pub fn run(code: String) -> String;
    }

    pub use lib::Code;
}

struct ToCors<T>(pub T);
impl<'r, T: Responder<'r, 'static>> Responder<'r, 'static> for ToCors<T> {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let mut response = self.0.respond_to(req)?;
        response.set_header(rocket::http::Header::new(
            "Access-Control-Allow-Origin",
            "*",
        ));
        response.set_header(rocket::http::Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, OPTIONS",
        ));
        response.set_header(rocket::http::Header::new(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        ));

        println!("Response: {:?}", response);

        Ok(response)
    }
}

// curl -X POST -H "Content-Type: application/json" -d {\"code\":\"print\"} http://localhost:1080/
#[post("/", format = "json", data = "<code>")]
fn index(code: String) -> ToCors<RawJson<String>> {
    ToCors(RawJson(hot_lib::run(code)))
}

#[options("/")]
fn options_preflight() -> ToCors<&'static str> {
    ToCors("")
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
        .mount("/", routes![index, options_preflight])
}
