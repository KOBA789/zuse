use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    web_sys::console::log_1(&"hello world".into());
    Ok(())
}
