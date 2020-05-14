use ttl_cache::TtlCache;

use std::collections::vec_deque::VecDeque;
use std::sync::RwLock;
use std::time::Duration;

use crate::types::SavedAlbum;

pub struct Cache {
    cache: RwLock<TtlCache<String, VecDeque<SavedAlbum>>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(TtlCache::new(12)),
        }
    }

    pub fn get_next(&self, key: &String) -> Option<SavedAlbum> {
        self.cache.write().ok().and_then(|mut entries| {
            entries.get_mut(key).and_then(|saved_albums| {
                saved_albums.pop_front().map(|saved_album| {
                    saved_albums.push_back(saved_album.clone());

                    saved_album
                })
            })
        })
    }

    pub fn insert(&self, key: String, mut value: Vec<SavedAlbum>) {
        self.cache.write().ok().map(move |mut entries| {
            entries.insert(
                key,
                value.drain(..).collect(),
                Duration::from_secs(30 * 60),
            )
        });
    }
}
