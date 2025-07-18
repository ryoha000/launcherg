use std::marker::PhantomData;

use derive_new::new;
use serde::{Deserialize, Serialize};

pub mod all_game_cache;
pub mod collection;
pub mod distance;
pub mod explored_cache;
pub mod file;
pub mod network;
pub mod process;
pub mod pubsub;

pub mod explorer;
pub mod repository;
pub mod windows;

#[derive(new, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Id<T> {
    pub value: i32,
    _marker: PhantomData<T>,
}
