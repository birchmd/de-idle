use {
    wasm_bindgen::{JsCast, JsValue, closure::Closure},
    web_sys::{Document, Element, HtmlElement, HtmlSelectElement},
};

pub mod actor;
pub mod dashboard;
pub mod plot;
pub mod tabs;

pub fn create_button<F: FnMut(&Element) + 'static>(
    document: &Document,
    text: &str,
    mut onclick: F,
) -> Result<HtmlElement, JsValue> {
    let button = document.create_element("button")?;
    button.set_text_content(Some(text));
    let local_button = button.clone();
    let closure = Closure::new::<Box<dyn FnMut()>>(Box::new(move || onclick(&local_button)));
    let elem: HtmlElement = button.unchecked_into();
    elem.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    Ok(elem)
}

pub fn create_drop_down<F: FnMut(String) + 'static>(
    document: &Document,
    default: &str,
    options: &[&str],
    mut onchange: F,
) -> Result<HtmlElement, JsValue> {
    let select = document.create_element("select")?;
    for option in options {
        let elem = document.create_element("option")?;
        elem.set_text_content(Some(*option));
        elem.set_attribute("value", option)?;
        select.append_child(&elem)?;
    }

    let select: HtmlElement = select.unchecked_into();
    let closure_select: HtmlSelectElement = select.clone().unchecked_into();
    closure_select.set_value(default);
    let closure =
        Closure::new::<Box<dyn FnMut()>>(Box::new(move || onchange(closure_select.value())));
    select.set_onchange(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    Ok(select)
}
