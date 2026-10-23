use anyhow::Context;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Default, Serialize, Deserialize)]
struct State {
    completed_path: Option<PathBuf>,
    last_error: Option<String>,
}

/// A durable receipt and a process lock make a lost Native Messaging response retryable.
pub(super) struct DownloadJob {
    _lock: File,
    state_path: PathBuf,
    state: State,
}

impl DownloadJob {
    pub fn open(
        root: &Path,
        request_id: &str,
        work_id: &str,
        items: &[String],
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !request_id.is_empty() && !items.is_empty(),
            "ダウンロード情報が空です"
        );
        let dir = root.join("download-jobs");
        fs::create_dir_all(&dir)?;
        let mut ordered_items = items.to_vec();
        ordered_items.sort();
        let identity = serde_json::to_vec(&(request_id, work_id, ordered_items))?;
        let key = format!("{:x}", Sha256::digest(identity));
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join(format!("{key}.lock")))?;
        lock.try_lock_exclusive()
            .context("このダウンロードは処理中です。しばらく待って再試行してください")?;
        let state_path = dir.join(format!("{key}.json"));
        let state = match fs::read(&state_path) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => State::default(),
            Err(error) => return Err(error.into()),
        };
        Ok(Self {
            _lock: lock,
            state_path,
            state,
        })
    }
    pub fn completed_path(&self) -> Option<PathBuf> {
        self.state.completed_path.clone()
    }
    pub fn record(&mut self, result: &anyhow::Result<PathBuf>) -> anyhow::Result<()> {
        self.state = match result {
            Ok(path) => State {
                completed_path: Some(path.clone()),
                last_error: None,
            },
            Err(error) => State {
                completed_path: None,
                last_error: Some(format!("{error:#}")),
            },
        };
        let mut temp = tempfile::NamedTempFile::new_in(self.state_path.parent().unwrap())?;
        temp.write_all(&serde_json::to_vec(&self.state)?)?;
        temp.as_file().sync_all()?;
        temp.persist(&self.state_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_job_重複実行を防ぎ完了と失敗を再起動後も保持する() {
        let root = tempfile::tempdir().unwrap();
        let items = vec!["game.zip".to_string()];
        let mut job = DownloadJob::open(root.path(), "request", "work", &items).unwrap();
        assert!(DownloadJob::open(root.path(), "request", "work", &items).is_err());
        job.record(&Err(anyhow::anyhow!("disk full"))).unwrap();
        drop(job);
        let mut job = DownloadJob::open(root.path(), "request", "work", &items).unwrap();
        assert!(job.completed_path().is_none());
        job.record(&Ok(root.path().join("installed"))).unwrap();
        drop(job);
        assert_eq!(
            DownloadJob::open(root.path(), "request", "work", &items)
                .unwrap()
                .completed_path(),
            Some(root.path().join("installed"))
        );
    }
}
