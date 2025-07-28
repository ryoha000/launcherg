use derive_new::new;
use std::marker::PhantomData;

use crate::domain::{
    explorer::{file::FileExplorer},
    file::File,
};

#[derive(new)]
pub struct ExplorerImpl<T> {
    _marker: PhantomData<T>,
}

pub struct Explorers {
    file_explorer: ExplorerImpl<File>,
}
pub trait ExplorersExt {
    type FileExplorer: FileExplorer;

    fn file_explorer(&self) -> &Self::FileExplorer;
}


impl ExplorersExt for Explorers {
    type FileExplorer = ExplorerImpl<File>;

    fn file_explorer(&self) -> &Self::FileExplorer {
        &self.file_explorer
    }
}

impl Explorers {
    pub fn new() -> Self {
        let file_explorer = ExplorerImpl::new();

        Self {
            file_explorer,
        }
    }
}
