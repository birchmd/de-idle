use {
    crate::ui::tabs::TabsBuilder,
    wasm_bindgen::{JsCast, JsValue},
    web_sys::{Document, Element},
};

pub fn create_goals_tab(
    document: &Document,
    tabs: &mut TabsBuilder,
) -> Result<Vec<Element>, JsValue> {
    let tab_content = document.create_element("div")?;
    tab_content.set_class_name("tabcontent");

    let table = document.create_element("table")?;
    tab_content.append_child(&table)?;

    let goals = [
        ("Going Steady", "horizontal.png"),
        ("The Slope is OVER 9000!", "vertical.png"),
        ("Step Up", "step.png"),
        ("Running Up That Hill", "slope.png"),
        ("What Goes Up Must Come Down", "neg_slope.png"),
    ];

    let mut checkboxes = Vec::with_capacity(goals.len());
    for (name, image) in goals {
        let checkbox = add_goal(document, &table, name, image)?;
        checkboxes.push(checkbox);
    }

    tabs.with("Goals".into(), tab_content.unchecked_into())?;

    Ok(checkboxes)
}

fn add_goal(
    document: &Document,
    table: &Element,
    name: &str,
    image: &str,
) -> Result<Element, JsValue> {
    let row = document.create_element("tr")?;
    table.append_child(&row)?;

    let checkbox = document.create_element("td")?;
    checkbox.set_text_content(Some("☐"));
    row.append_child(&checkbox)?;

    let cell = document.create_element("td")?;
    cell.set_text_content(Some(name));
    row.append_child(&cell)?;

    let cell = document.create_element("td")?;
    let img = document.create_element("img")?;
    img.set_class_name("goal-image");
    img.set_attribute("src", image)?;
    cell.append_child(&img)?;
    row.append_child(&cell)?;

    Ok(checkbox)
}
