use chashmap::CHashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use reqwest::blocking::Client;
use rocket::http::{Cookie, Cookies};
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;

use crate::cache::Cache;
use crate::oauth::UserState;
use crate::types::{AlbumJson, Paging, SavedAlbum};

use std::collections::HashMap;

#[get("/random_album")]
pub fn next_random_album(
    oauth_tokens: State<CHashMap<String, UserState>>,
    album_cache: State<Cache>,
    mut cookies: Cookies,
) -> Result<Template, Redirect> {
    cookies
        .get_private("temp_username")
        .and_then(|username| {
            oauth_tokens
                .get_mut(username.value())
                .map(|mut user_state| get_albums(username.value(), &album_cache, &mut user_state))
        })
        .and_then(|mut saved_albums| {
            if saved_albums.is_empty() {
                return Some(Template::render("empty", HashMap::<String, String>::new()));
            }

            let albums = saved_albums
                .drain(..)
                .map(|saved_album| AlbumJson::from(saved_album))
                .collect::<Vec<AlbumJson>>();

            serde_json::to_string(&albums).ok().map(|albums_str| {
                let mut context = HashMap::new();

                context.insert("albums", albums_str);

                Template::render("random_album", context)
            })
        })
        .ok_or_else(|| {
            cookies.remove_private(Cookie::named("temp_username"));

            Redirect::found("/")
        })
}

fn get_albums(
    username: &str,
    album_cache: &State<Cache>,
    user_state: &mut UserState,
) -> Vec<SavedAlbum> {
    let mut albums = album_cache.get(&username.to_string()).unwrap_or_else(|| {
        let mut albums = Vec::new();

        let client = Client::new();
        let mut next_url_opt = Some("https://api.spotify.com/v1/me/albums".to_string());
        while let Some(next_url) = next_url_opt {
            next_url_opt = user_state.token_checked().ok().and_then(|token| {
                client
                    .get(&next_url[..])
                    .bearer_auth(token)
                    .send()
                    .ok()
                    .and_then(|response| response.json::<Paging<SavedAlbum>>().ok())
                    .and_then(|mut page| {
                        albums.extend(page.items.drain(..));

                        page.next
                    })
            });
        }

        album_cache.insert(username.to_string(), albums.clone());

        albums
    });

    let mut rng = thread_rng();
    albums.shuffle(&mut rng);

    albums.drain(..144).collect::<Vec<SavedAlbum>>()
}
