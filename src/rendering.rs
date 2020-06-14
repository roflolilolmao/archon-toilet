use handlebars::{Handlebars, TemplateRenderError};
use serde_json::json;

use crate::database::Table;

pub fn render(tables: Vec<Table>, template: &str) -> Result<String, TemplateRenderError> {
    Handlebars::new().render_template(template, &json!({ "tables": &tables }))
}
