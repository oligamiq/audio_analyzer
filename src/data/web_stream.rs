use super::RawDataStreamLayer;
// use egui::mutex::Mutex;
use std::sync::Mutex;
use std::sync::{Arc, OnceLock};
use wasm_bindgen::JsValue;
use wasm_bindgen::{prelude::Closure, JsCast as _};
use wasm_bindgen_futures::JsFuture;
// use web_sys::js_sys::Reflect;
// use wasm_thread as thread;

// https://zenn.dev/tetter/articles/web-realtime-audio-processing
// https://qiita.com/okaxaki/items/c807bdfe3e96d6ef7960

pub struct WebAudioStream(pub Arc<Mutex<Vec<f32>>>);

impl WebAudioStream {
    pub fn new() -> Self {
        Self(ON_WEB_STRUCT.get().unwrap().clone())
    }
}

impl RawDataStreamLayer for WebAudioStream {
    fn try_recv(&mut self) -> Option<Vec<f32>> {
        let mut data = self.0.lock().unwrap();

        if data.is_empty() {
            return None;
        }

        let data = data.drain(..).collect();

        Some(data)
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE.get().unwrap().clone()
    }

    fn start(&mut self) {
        // Do nothing
    }
}

static ON_WEB_STRUCT: OnceLock<Arc<Mutex<Vec<f32>>>> = OnceLock::new();
static SAMPLE_RATE: OnceLock<u32> = OnceLock::new();

pub async fn init_on_web_struct() {
    let on_web = OnWebStruct::new().await;

    let vec = on_web.data.clone();

    ON_WEB_STRUCT.get_or_init(|| vec);

    let sample_rate = on_web.sample_rate.unwrap();

    SAMPLE_RATE.get_or_init(|| sample_rate);

    core::mem::forget(on_web);

    // assert!(ON_WEB_STRUCT.get().is_none());

    // thread::spawn(|| {
    //     spawn_local(async move {
    //         let on_web = OnWebStruct::new();

    //         let on_web = on_web.await;

    //         panic!("-2");

    //         let vec = on_web.data.clone();

    //         panic!("-1");

    //         ON_WEB_STRUCT.get_or_init(|| vec);

    //         panic!("0");

    //         let sample_rate = on_web.sample_rate.unwrap();

    //         SAMPLE_RATE.get_or_init(|| sample_rate);

    //         core::mem::forget(on_web);

    //         panic!("1");

    //         // sender.send(()).unwrap();
    //     });
    // });
}

pub struct OnWebStruct {
    pub data: Arc<Mutex<Vec<f32>>>,
    sample_rate: Option<u32>,
    _audio_ctx: web_sys::AudioContext,
    _source: web_sys::MediaStreamAudioSourceNode,
    _media_devices: web_sys::MediaDevices,
    _stream: web_sys::MediaStream,
    _js_closure: Closure<dyn FnMut(wasm_bindgen::JsValue)>,
    _worklet_node: web_sys::AudioWorkletNode,
}

impl std::fmt::Debug for OnWebStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OnWebStruct")
    }
}

impl OnWebStruct {
    pub async fn new() -> Self {
        // let audio_ctx = web_sys::AudioContext::new().unwrap();
        let config = web_sys::AudioContextOptions::new();
        config.set_sample_rate(44100.0);

        let audio_ctx = web_sys::AudioContext::new_with_context_options(&config).unwrap();

        let media_devices = web_sys::window()
            .unwrap()
            .navigator()
            .media_devices()
            .unwrap();

        let constraints = web_sys::MediaStreamConstraints::new();

        let js_true = wasm_bindgen::JsValue::from(true);

        constraints.set_audio(&js_true);

        let stream = media_devices
            .get_user_media_with_constraints(&constraints)
            .unwrap();

        let stream = JsFuture::from(stream).await.unwrap();

        // panic!("## 2 {:?}", stream);

        let stream = stream.dyn_into::<web_sys::MediaStream>().unwrap();

        // panic!("3 {:?}", stream);

        let source = audio_ctx.create_media_stream_source(&stream).unwrap();

        // panic!("4 {:?}", source);

        // Return about Float32Array
        // return first input's first channel's samples
        // https://developer.mozilla.org/ja/docs/Web/API/AudioWorkletProcessor/process
        let processor_js_code = r#"
            class MyProcessor extends AudioWorkletProcessor {
                constructor() {
                    super();

                    console.log('MyProcessor is constructed');
                }

                process(inputs, outputs, parameters) {
                    this.port.postMessage(Float32Array.from(inputs[0][0]));

                    console.log('MyProcessor is processing');

                    return true;
                }
            }

            registerProcessor('my-processor', MyProcessor);

            console.log('MyProcessor is registered');
        "#;

        // 明示的にresumeを呼ぶ
        JsFuture::from(audio_ctx.resume().unwrap()).await.unwrap();

        let processor = audio_ctx
            .audio_worklet()
            .expect("Failed to get audio worklet")
            .add_module(&format!("data:application/javascript,{processor_js_code}"))
            .unwrap();

        JsFuture::from(processor).await.unwrap();

        let worklet_node = web_sys::AudioWorkletNode::new(&audio_ctx, "my-processor").expect(
            "Failed to create \
                                                                               audio worklet node",
        );

        web_sys::console::log_1(&JsValue::from(&worklet_node));

        source.connect_with_audio_node(&worklet_node).unwrap();

        web_sys::console::log_1(&JsValue::from(&source));

        // panic!("## 5 {:?}", worklet_node);

        let data = Arc::new(Mutex::new(Vec::new()));

        let data_clone = data.clone();

        // Float32Array
        let js_closure = Closure::wrap(Box::new(move |msg: wasm_bindgen::JsValue| {
            web_sys::console::log_1(&JsValue::from("onmessage"));
            web_sys::console::log_1(&msg);

            let msg_event = msg.dyn_into::<web_sys::MessageEvent>().unwrap();

            let data = msg_event.data();

            let data: Vec<f32> = serde_wasm_bindgen::from_value(data).unwrap();

            let mut data_clone = data_clone.lock().unwrap();

            data_clone.extend(data);
        }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);

        let js_func = js_closure.as_ref().unchecked_ref();

        worklet_node
            .port()
            .expect("Failed to get port")
            .set_onmessage(Some(js_func));

        let destination = audio_ctx.create_media_stream_destination().unwrap();

        worklet_node.connect_with_audio_node(&destination).unwrap();

        let processed_stream = destination.stream();

        web_sys::console::log_1(&JsValue::from(&processed_stream));

        web_sys::console::log_1(&JsValue::from(&worklet_node));

        // worklet_node.connect_with_audio_node(&source).unwrap();

        web_sys::console::log_1(&JsValue::from(&worklet_node));

        web_sys::console::log_1(&JsValue::from(&source));

        // jsのグローバルに持たせる

        // let global = web_sys::window().unwrap();

        // set worklet_node
        // Reflect::set(
        //     &global,
        //     &"on_web_struct".into(),
        //     &JsValue::from(&worklet_node),
        // )
        // .unwrap();

        // set audio_ctx
        // Reflect::set(
        //     &global,
        //     &"on_web_struct_audio_ctx".into(),
        //     &JsValue::from(&audio_ctx),
        // )
        // .unwrap();

        // set source
        // Reflect::set(
        //     &global,
        //     &"on_web_struct_source".into(),
        //     &JsValue::from(&source),
        // )
        // .unwrap();

        // set stream
        // Reflect::set(
        //     &global,
        //     &"on_web_struct_stream".into(),
        //     &JsValue::from(&stream),
        // )
        // .unwrap();

        // set media_devices
        // Reflect::set(
        //     &global,
        //     &"on_web_struct_media_devices".into(),
        //     &JsValue::from(&media_devices),
        // )
        // .unwrap();

        // set js_closure
        // Reflect::set(
        //     &global,
        //     &"on_web_struct_js_closure".into(),
        //     &JsValue::from(js_func),
        // )
        // .unwrap();

        // panic!("## 6");

        OnWebStruct {
            data,
            sample_rate: Some(44100),
            _audio_ctx: audio_ctx,
            _source: source,
            _media_devices: media_devices,
            _stream: stream,
            _js_closure: js_closure,
            _worklet_node: worklet_node,
        }
    }
}

impl Drop for OnWebStruct {
    fn drop(&mut self) {
        let _ = self._audio_ctx.close();
    }
}
