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
            cache: RwLock::new(TtlCache::new(12 * 12)),
        }
    }

    pub fn get(&self, key: &String) -> Option<Vec<SavedAlbum>> {
        self.cache.read().ok().and_then(|entries| {
            entries.get(key).map(|vec_deque| {
                vec_deque.iter().cloned().collect::<Vec<SavedAlbum>>()
            })
        })
    }

    pub fn insert(&self, key: String, mut value: Vec<SavedAlbum>) {
        self.cache.write().ok().map(move |mut entries| {
            entries.insert(
                key,
                value.drain(..).collect(),
                Duration::from_secs(12 * 60 * 60),
            )
        });
    }
}
