use {
    std::collections::HashMap,
    wasm_bindgen::{JsCast, JsValue, closure::Closure},
    web_sys::{Document, Element, HtmlElement},
};

pub struct TabsBuilder<'a> {
    document: &'a Document,
    this: HtmlElement,
    content: HashMap<String, (HtmlElement, HtmlElement)>,
    order: Vec<String>,
}

impl<'a> TabsBuilder<'a> {
    pub fn new(document: &'a Document) -> Result<Self, JsValue> {
        let this = document.create_element("div")?;
        this.set_class_name("tab");
        Ok(Self {
            document,
            this: this.unchecked_into(),
            content: Default::default(),
            order: Vec::new(),
        })
    }

    pub fn with(&mut self, name: String, content: HtmlElement) -> Result<Element, JsValue> {
        let button = self.document.create_element("button")?;
        button.set_class_name("tablinks");
        button.set_text_content(Some(&name));

        self.content
            .insert(name.clone(), (content, button.clone().unchecked_into()));
        self.order.push(name);

        Ok(button)
    }

    pub fn build(&self, body: &HtmlElement) -> Result<(), JsValue> {
        let do_select =
            move |selected_name: &str,
                  closure_content: &HashMap<String, (HtmlElement, HtmlElement)>| {
                for (name, (element, button)) in closure_content {
                    if name == selected_name {
                        element.style().set_property("display", "grid").ok();
                        button.set_class_name("tablinks active");
                        button.set_text_content(Some(name));
                    } else {
                        element.style().set_property("display", "none").ok();
                        button.set_class_name("tablinks");
                    }
                }
            };

        body.append_child(&self.this)?;
        for name in &self.order {
            let (element, button) = self.content.get(name).ok_or(JsValue::undefined())?;
            let name = name.clone();
            let closure_content = self.content.clone();
            let closure = Closure::new::<Box<dyn FnMut()>>(Box::new(move || {
                do_select(&name, &closure_content)
            }));
            button.set_onclick(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
            self.this.append_child(button)?;
            body.append_child(element)?;
        }

        Ok(())
    }
}
