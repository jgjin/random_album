use serde::Deserialize;

use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Paging<T> {
    pub items: Vec<T>,
    pub next: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct SavedAlbum {
    pub album: Album,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Album {
    pub artists: Vec<Artist>,
    pub images: Vec<Image>,
    pub name: String,
    pub external_urls: HashMap<String, String>,
    pub copyrights: Vec<Copyright>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Artist {
    pub name: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Image {
    pub url: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Copyright {
    pub text: String,
    #[serde(rename = "type")]
    pub copyright_type: String,
}
