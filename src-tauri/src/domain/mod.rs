use std::marker::PhantomData;

use derive_new::new;
use serde::{Deserialize, Serialize};

pub mod collection;
pub mod distance;
pub mod file;
pub mod network;

pub mod explorer;
pub mod repository;

#[derive(new, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Id<T> {
    pub value: i32,
    _marker: PhantomData<T>,
}
