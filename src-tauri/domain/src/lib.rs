use std::marker::PhantomData;

use derive_new::new;
use serde::{Deserialize, Serialize};

pub mod all_game_cache;
pub mod collection;
pub mod distance;
pub mod dmm_work_pack;
pub mod erogamescape;
pub mod explored_cache;
pub mod extension;
pub mod file;
pub mod game_matcher;
pub mod icon;
pub mod native_host_log;
pub mod network;
pub mod process;
pub mod pubsub;
pub mod save_image_queue;
pub mod thumbnail;
pub mod work_download_path;
pub mod work_omit;
pub mod work_parent_pack;
pub mod works;

pub mod repository;
pub mod scan;
pub mod service;
pub mod windows;

#[derive(new, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Id<T> {
    pub value: i32,
    _marker: PhantomData<T>,
}

impl<T> core::cmp::PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> core::cmp::Eq for Id<T> {}

impl<T> core::hash::Hash for Id<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(new, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StrId<T> {
    pub value: String,
    _marker: PhantomData<T>,
}
