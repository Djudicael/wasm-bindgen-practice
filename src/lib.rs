use js_sys::Array;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlElement, Request, RequestInit, RequestMode, Response};

use crate::{
    canvas::run_canvas,
    defined_in_js::MyClass,
    pass_closure::{setup_clicker, setup_clock},
    performance::perf_to_system,
    req_frame::frame_run,
    wasm_in_wasm::run_async,
    web_gl::webgl,
    webrtc::web_start,
    websocket::start_websocket, paint::start_paint,
};
pub mod counter;
pub mod defined_in_js;
mod pass_closure;
mod wasm_in_wasm;
use wasm_bindgen_futures::{spawn_local, JsFuture};
pub mod canvas;
pub mod fm_oscillator;
mod paint;
mod performance;
mod req_frame;
mod web_gl;
mod webrtc;
mod websocket;
pub mod web_worker;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}
fn bare_bones() {
    log("Hello from Rust!");
    log_u32(42);
    log_many("Logging", "many values!");
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn using_a_macro() {
    console_log!("Hello {}!", "world");
    console_log!("Let's print some numbers...");
    console_log!("1 + 3 = {}", 1 + 3);
}

// And finally, we don't even have to define the `log` function ourselves! The
// `web_sys` crate already has it defined for us.

fn using_web_sys() {
    use web_sys::console;

    console::log_1(&"Hello using web-sys".into());

    let js: JsValue = 4.into();
    console::log_2(&"Logging arbitrary values looks like".into(), &js);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust!");

    body.append_child(&val)?;

    bare_bones();
    using_a_macro();
    using_web_sys();

    let x = MyClass::new();
    assert_eq!(x.number(), 42);
    x.set_number(10);
    log(&x.render());

    spawn_local(async {
        run_async().await.unwrap_throw();
    });

    // Pass closure
    // One of the first interesting things we can do with closures is simply
    // access stack data in Rust!
    let array = Array::new();
    array.push(&"Hello".into());
    array.push(&1.into());
    let mut first_item = None;
    array.for_each(&mut |obj, idx, _arr| match idx {
        0 => {
            assert_eq!(obj, "Hello");
            first_item = obj.as_string();
        }
        1 => assert_eq!(obj, 1),
        _ => panic!("unknown index: {}", idx),
    });
    assert_eq!(first_item, Some("Hello".to_string()));

    // Below are some more advanced usages of the `Closure` type for closures
    // that need to live beyond our function call.

    setup_clock(&window, &document)?;
    setup_clicker(&document);

    // And now that our demo is ready to go let's switch things up so
    // everything is displayed and our loading prompt is hidden.
    document
        .get_element_by_id("loading")
        .expect("should have #loading on the page")
        .dyn_ref::<HtmlElement>()
        .expect("#loading should be an `HtmlElement`")
        .style()
        .set_property("display", "none")?;
    document
        .get_element_by_id("script")
        .expect("should have #script on the page")
        .dyn_ref::<HtmlElement>()
        .expect("#script should be an `HtmlElement`")
        .style()
        .set_property("display", "block")?;

    //performance
    let performance = window
        .performance()
        .expect("performance should be available");
    console_log!("the current time (in ms) is {}", performance.now());

    let start = perf_to_system(performance.timing().request_start());
    let end = perf_to_system(performance.timing().response_end());

    console_log!("request started at {}", humantime::format_rfc3339(start));
    console_log!("request ended at {}", humantime::format_rfc3339(end));

    // canvas
    run_canvas(&document);

    //webGl
    // webgl(&document)?;

    //websocket
    start_websocket()?;
    //webrtc
    // web_start();

    //frame
    // frame_run()?;

    // simple paint

    start_paint()?;

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[wasm_bindgen]
pub async fn run_fetch(repo: String) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = format!("https://api.github.com/repos/{}/branches/master", repo);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    request
        .headers()
        .set("Accept", "application/vnd.github.v3+json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // Send the JSON response back to JS.
    Ok(json)
}
