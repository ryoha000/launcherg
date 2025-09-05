use std::fs;
use std::path::{Path, PathBuf};

use domain::scan::{FileSystem, WorkCandidate, CandidateKind};
use domain::explored_cache::ExploredCache;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct LocalFileSystem;

impl LocalFileSystem {
    fn kind_for(path: &Path) -> CandidateKind {
        let lower = path.to_string_lossy().to_ascii_lowercase();
        // ディレクトリ優先（拡張子風のディレクトリ名は Folder として扱う）
        if path.is_dir() { return CandidateKind::Folder; }
        if lower.ends_with(".exe") { return CandidateKind::Exe; }
        if lower.ends_with(".lnk") { return CandidateKind::Shortcut; }
        CandidateKind::Other
    }
}

impl FileSystem for LocalFileSystem {
    fn walk_dir(&self, roots: &[PathBuf], exclude: Option<Arc<ExploredCache>>) -> anyhow::Result<Box<dyn Iterator<Item = WorkCandidate> + Send>> {
        // ストリーミングで返す（全件 collect しない）
        let roots_vec: Vec<PathBuf> = roots.iter().cloned().collect();
        let iter = roots_vec
            .into_iter()
            .filter(|root| root.exists())
            .flat_map(move |root| {
                let exclude = exclude.clone();
                walkdir::WalkDir::new(root)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter_map(move |entry| {
                        let path = entry.path().to_path_buf();
                        if let Some(ref set) = exclude {
                            let s = path.to_string_lossy().to_string();
                            if set.contains(&s) {
                                return None;
                            }
                        }
                        let kind = Self::kind_for(&path);
                        match kind {
                            CandidateKind::Exe | CandidateKind::Shortcut => Some(WorkCandidate::new(path, kind)),
                            CandidateKind::Other | CandidateKind::Folder => None,
                        }
                    })
            });
        Ok(Box::new(iter))
    }

    fn stat(&self, path: &Path) -> anyhow::Result<std::fs::Metadata> { Ok(fs::metadata(path)?) }

}


