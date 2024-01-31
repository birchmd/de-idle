use wasm_bindgen::JsValue;

mod game_state;

fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");
    let label = document.create_element("p")?;
    label.set_text_content(Some("Hello, World!"));
    body.append_child(&label)?;
    Ok(())
}
