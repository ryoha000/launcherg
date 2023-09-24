use derive_new::new;
use std::marker::PhantomData;

use crate::domain::{
    explorer::{file::FileExplorer, network::NetworkExplorer},
    file::File,
    network::NetWork,
};

#[derive(new)]
pub struct ExplorerImpl<T> {
    _marker: PhantomData<T>,
}

pub struct Explorers {
    file_explorer: ExplorerImpl<File>,
    network_explorer: ExplorerImpl<NetWork>,
}
pub trait ExplorersExt {
    type FileExplorer: FileExplorer;
    type NetworkExplorer: NetworkExplorer;

    fn file_explorer(&self) -> &Self::FileExplorer;
    fn network_explorer(&self) -> &Self::NetworkExplorer;
}

impl ExplorersExt for Explorers {
    type FileExplorer = ExplorerImpl<File>;
    type NetworkExplorer = ExplorerImpl<NetWork>;

    fn file_explorer(&self) -> &Self::FileExplorer {
        &self.file_explorer
    }
    fn network_explorer(&self) -> &Self::NetworkExplorer {
        &self.network_explorer
    }
}

impl Explorers {
    pub fn new() -> Self {
        let file_explorer = ExplorerImpl::new();
        let network_explorer = ExplorerImpl::new();

        Self {
            file_explorer,
            network_explorer,
        }
    }
}
