use rocket_contrib::templates::Template;

use std::collections::HashMap;

#[get("/error")]
pub fn error() -> Template {
    Template::render("error", HashMap::<String, String>::new())
}
