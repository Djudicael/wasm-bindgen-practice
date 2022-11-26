use js_sys::Date;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{Document, Element, HtmlElement, Window};

// Set up a clock on our page and update it each second to ensure it's got
// an accurate date.
//
// Note the usage of `Closure` here because the closure is "long lived",
// basically meaning it has to persist beyond the call to this one function.
// Also of note here is the `.as_ref().unchecked_ref()` chain, which is how
// you can extract `&Function`, what `web-sys` expects, from a `Closure`
// which only hands you `&JsValue` via `AsRef`.pub 
pub fn setup_clock(window: &Window, document: &Document) -> Result<(), JsValue> {
    let current_time = document
        .get_element_by_id("current-time")
        .expect("should have #current-time on the page");
    update_time(&current_time);
    let a = Closure::<dyn Fn()>::new(move || update_time(&current_time));
    window
        .set_interval_with_callback_and_timeout_and_arguments_0(a.as_ref().unchecked_ref(), 1000)?;
    fn update_time(current_time: &Element) {
        current_time.set_inner_html(&String::from(
            Date::new_0().to_locale_string("en-GB", &JsValue::undefined()),
        ));
    }

    // The instance of `Closure` that we created will invalidate its
    // corresponding JS callback whenever it is dropped, so if we were to
    // normally return from `setup_clock` then our registered closure will
    // raise an exception when invoked.
    //
    // Normally we'd store the handle to later get dropped at an appropriate
    // time but for now we want it to be a global handler so we use the
    // `forget` method to drop it without invalidating the closure. Note that
    // this is leaking memory in Rust, so this should be done judiciously!
    a.forget();

    Ok(())
}

// We also want to count the number of times that our green square has been
// clicked. Our callback will update the `#num-clicks` div.
//
// This is pretty similar above, but showing how closures can also implement
// `FnMut()`.
pub fn setup_clicker(document: &Document) {
    let num_clicks = document
        .get_element_by_id("num-clicks")
        .expect("should have #num-clicks on the page");
    let mut clicks = 0;
    let a = Closure::<dyn FnMut()>::new(move || {
        clicks += 1;
        num_clicks.set_inner_html(&clicks.to_string());
    });
    document
        .get_element_by_id("green-square")
        .expect("should have #green-square on the page")
        .dyn_ref::<HtmlElement>()
        .expect("#green-square be an `HtmlElement`")
        .set_onclick(Some(a.as_ref().unchecked_ref()));

    // See comments in `setup_clock` above for why we use `a.forget()`.
    a.forget();
}
