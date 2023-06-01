use async_trait::async_trait;

use crate::domain::file::LnkMetadata;

#[async_trait]
pub trait FileExplorer {
    async fn get_file_paths_by_exts(
        &self,
        explorer_dir_path: String,
        filter_exts: Vec<String>,
    ) -> anyhow::Result<Vec<String>>;
    async fn get_lnk_source_paths(
        &self,
        lnk_file_paths: Vec<String>,
    ) -> anyhow::Result<Vec<LnkMetadata>>;
}
