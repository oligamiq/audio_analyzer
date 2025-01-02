cargo watch -w "rocket-webapi/lib" -x 'build -p lib'
trunk serve -p 880

cargo run -p reqwest-client
cargo b -r -p rocket-webapi
C:\Users\oligami\audio_analyzer\target\release\rocket-webapi.exe
