use std::future::Future;

pub fn save_file(data: Vec<u8>) {
    let task = rfd::AsyncFileDialog::new()
        .set_file_name("audio_analyzer_save.json")
        .save_file();

    execute(async move {
        let file = task.await;
        if let Some(file) = file {
            _ = file.write(&data).await;
        }
    });
}

pub fn open_file<F: Fn(Vec<u8>) + 'static + Send>(callback: F) {
    let task = rfd::AsyncFileDialog::new().pick_file();

    execute(async move {
        let file = task.await;
        if let Some(file) = file {
            let data = file.read().await;

            callback(data);
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
