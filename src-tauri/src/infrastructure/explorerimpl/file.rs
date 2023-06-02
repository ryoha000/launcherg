use async_trait::async_trait;

use crate::domain::{explorer::file::FileExplorer, file::File};

use super::explorer::ExplorerImpl;

#[async_trait]
impl FileExplorer for ExplorerImpl<File> {}
