use std::io::prelude::*;
use std::sync::Arc;

use base64::{engine::general_purpose, Engine as _};
use derive_new::new;
use domain::service::save_path_resolver::SavePathResolver;
use domain::works::Work;
use domain::StrId;

use domain::file::PlayHistory;

#[derive(new)]
pub struct FileUseCase {
    resolver: Arc<dyn SavePathResolver>,
}

impl FileUseCase {
    pub fn get_new_upload_image_path(&self, id: i32) -> anyhow::Result<String> {
        // resolver.memos_dir() 配下に UUID.png を生成
        Ok(self.resolver.memo_image_new_png_path(&id.to_string()))
    }
    pub async fn upload_image(&self, id: i32, base64_image: String) -> anyhow::Result<String> {
        let path = self.get_new_upload_image_path(id)?;
        let decoded_data = general_purpose::STANDARD_NO_PAD.decode(base64_image)?;

        let mut file = std::fs::File::create(&path)?;
        file.write_all(&decoded_data)?;
        Ok(path)
    }
    pub fn get_play_time_minutes(&self, work_id: StrId<Work>) -> anyhow::Result<f32> {
        let path = self.resolver.play_history_jsonl_path(work_id);

        let exist = std::path::Path::new(&path).exists();
        if !exist {
            return Ok(0.0);
        }

        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);

        let mut histories = vec![];
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(history) = serde_json::from_str::<PlayHistory>(&line) {
                    histories.push(history)
                }
            }
        }

        Ok(histories.into_iter().map(|v| v.minutes).sum())
    }
}
