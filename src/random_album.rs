use chashmap::CHashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use reqwest::blocking::Client;
use rocket::http::Cookies;
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;

use crate::cache::Cache;
use crate::oauth::UserState;
use crate::types::{Paging, SavedAlbum};

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
                .and_then(|mut user_state| {
                    next_album(username.value(), &album_cache, &mut user_state)
                })
        })
        .map(|saved_album| {
            let album = saved_album.album;

            let mut context = HashMap::new();

            context.insert("album_title", Some(album.name));
            context.insert(
                "album_url",
                album.external_urls.get("spotify").map(|url| url.clone()),
            );
            context.insert(
                "artists",
                Some(
                    album
                        .artists
                        .iter()
                        .map(|artist| artist.name.clone())
                        .collect::<Vec<String>>()
                        .join(", "),
                ),
            );
            context.insert(
                "copyright",
                album
                    .copyrights
                    .iter()
                    .filter(|copyright| copyright.copyright_type == "C")
                    .next()
                    .or(album.copyrights.first())
                    .and_then(|copyright| match &copyright.copyright_type[..] {
                        "C" => {
                            let mut cleaned = copyright.text.replace("(C)", "©");
                            if !cleaned.starts_with("©") && !cleaned.starts_with("℗") {
                                cleaned = format!("© {}", cleaned);
                            }

                            Some(cleaned)
                        }
                        "P" => {
                            let mut cleaned = copyright.text.replace("(P)", "℗");
                            if !cleaned.starts_with("℗") && !cleaned.starts_with("©") {
                                cleaned = format!("℗ {}", cleaned);
                            }

                            Some(cleaned)
                        }
                        _ => None,
                    }),
            );
            context.insert(
                "image_url",
                album.images.first().map(|image| image.url.clone()),
            );

            Template::render("random_album", context)
        })
        .ok_or(Redirect::found("/error"))
}

fn next_album(
    username: &str,
    album_cache: &State<Cache>,
    user_state: &mut UserState,
) -> Option<SavedAlbum> {
    album_cache.get_next(&username.to_string()).or_else(|| {
        let client = Client::new();

        let mut albums = Vec::new();
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

        let mut rng = thread_rng();
        albums.shuffle(&mut rng);

        album_cache.insert(username.to_string(), albums);

        album_cache.get_next(&username.to_string())
    })
}
