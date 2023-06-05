use std::io::Write;
use std::{fs, path::Path};

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use uuid::Uuid;

use crate::{
    domain::{explorer::file::FileExplorer, file::File},
    infrastructure::util::get_save_root_abs_dir,
};

use super::explorer::ExplorerImpl;

const MEMOS_ROOT_DIR: &str = "game-memos";

#[async_trait]
impl FileExplorer for ExplorerImpl<File> {
    fn save_base64_image(&self, path: &str, data: String) -> anyhow::Result<()> {
        let decoded_data = general_purpose::STANDARD_NO_PAD.decode(data)?;

        let mut file = std::fs::File::create(path)?;
        file.write_all(&decoded_data)?;
        Ok(())
    }
    fn get_save_image_path(&self, id: i32) -> anyhow::Result<String> {
        let dir = Path::new(&get_save_root_abs_dir())
            .join(MEMOS_ROOT_DIR)
            .join(id.to_string());
        fs::create_dir_all(&dir).unwrap();
        Ok(Path::new(&dir)
            .join(format!("{}.png", Uuid::new_v4().to_string()))
            .to_string_lossy()
            .to_string())
    }
}
