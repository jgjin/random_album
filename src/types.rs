use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize)]
pub struct AlbumJson {
    pub artists: String,
    pub image_url: String,
    pub name: String,
    pub external_url: String,
    pub copyright: String,
}

impl From<SavedAlbum> for AlbumJson {
    fn from(saved_album: SavedAlbum) -> Self {
        Self {
            artists: saved_album
                .album
                .artists
                .iter()
                .map(|artist| artist.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
            image_url: saved_album
                .album
                .images
                .iter()
                .next()
                .map(|image| image.url.clone())
                .unwrap_or("".to_string()),
            name: saved_album.album.name,
            external_url: saved_album
                .album
                .external_urls
                .get("spotify")
                .map(|external_url| external_url.clone())
                .unwrap_or("".to_string()),
            copyright: saved_album
                .album
                .copyrights
                .iter()
                .filter(|copyright| copyright.copyright_type == "C")
                .next()
                .or(saved_album.album.copyrights.first())
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
                })
                .unwrap_or("".to_string()),
        }
    }
}
