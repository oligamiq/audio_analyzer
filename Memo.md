cargo watch -w "rocket-webapi/lib" -x 'build -p lib'
trunk serve -p 880

cargo run -p reqwest-client
cargo run -p audio_analyzer_inner_checker -r
cargo b -r -p rocket-webapi
C:\Users\oligami\audio_analyzer\target\release\rocket-webapi.exe
