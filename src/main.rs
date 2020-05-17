#![feature(proc_macro_hygiene, decl_macro)]

extern crate chashmap;
extern crate chrono;
extern crate dotenv;
extern crate oauth2;
extern crate rand;
extern crate reqwest;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate ttl_cache;

use chashmap::CHashMap;
use dotenv::dotenv;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

mod assets;
mod cache;
mod error;
mod oauth;
mod oauth_token;
mod random_album;
mod types;

fn main() {
    dotenv().ok();

    rocket::ignite()
        .attach(Template::fairing())
        .manage(CHashMap::<String, oauth::UserState>::new())
        .manage(cache::Cache::new())
        .mount("/public", StaticFiles::from("static"))
        .mount(
            "/",
            routes![
                assets::favicon,
                error::error,
                oauth::begin_oauth,
                oauth::end_oauth_accept,
                oauth::end_oauth_deny,
                random_album::next_random_album,
            ],
        )
        .launch();
}
