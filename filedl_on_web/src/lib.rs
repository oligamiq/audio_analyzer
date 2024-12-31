pub fn file_dl(data: &[u8], name: &str) {
    let array = js_sys::Array::new();
    let uint8arr = js_sys::Uint8Array::new(
        // Safety: No wasm allocations happen between creating the view and consuming it in the array.push
        &unsafe { js_sys::Uint8Array::view(&data) }.into(),
    );
    array.push(&uint8arr.buffer());
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &{
        let property = web_sys::BlobPropertyBag::new();
        property.set_type("application/octet-stream");
        property
    })
    .unwrap();
    let download_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    let window = web_sys::window().expect("Window not found");
    let document = window.document().expect("Document not found");

    let output_el = document.create_element("a").unwrap();
    let output: web_sys::HtmlAnchorElement = wasm_bindgen::JsCast::dyn_into(output_el).unwrap();

    output.set_href(&download_url);
    output.set_download(&name);

    output.click();

    // Clean up
    web_sys::Url::revoke_object_url(&download_url).unwrap();

    output.remove();
}
