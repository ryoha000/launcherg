use derive_new::new;
use std::path::{Path, PathBuf};

// データモデル（段階型）
#[derive(new, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkCandidate {
    pub path: PathBuf,
    pub kind: CandidateKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CandidateKind {
    Exe,
    Shortcut,
    Folder,
    Other,
}

#[derive(new, Clone, Debug, PartialEq)]
pub struct ResolvedWork {
    pub candidate: WorkCandidate,
    pub title: String,
    pub egs_id: i32,
    pub distance: f32,
}

#[derive(new, Clone, Debug, PartialEq)]
pub struct PersistedWork {
    pub id: i64,
    pub resolved: ResolvedWork,
}

#[derive(new, Clone, Debug)]
pub struct ScanStats {
    pub found: usize,
    pub recognized: usize,
    pub persisted: usize,
    pub skipped: usize,
    pub duplicates: usize,
}

#[derive(new, Clone, Debug)]
pub struct ScanCache;

#[derive(new, Clone, Debug)]
pub struct ScanContext {
    pub started_at: std::time::Instant,
    pub stats: ScanStats,
    pub cache: ScanCache,
}

// ドメイン ポート（トレイト）
#[trait_variant::make(Send)]
#[mockall::automock]
pub trait FileSystem {
    fn walk_dir(
        &self,
        roots: &[PathBuf],
        exclude: Option<std::sync::Arc<crate::explored_cache::ExploredCache>>,
    ) -> anyhow::Result<Box<dyn Iterator<Item = WorkCandidate> + Send>>;
    fn stat(&self, path: &Path) -> anyhow::Result<std::fs::Metadata>;
}

// GameRecognizer は廃止

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait MetadataExtractor {
    fn enrich(&self, c: WorkCandidate) -> anyhow::Result<WorkCandidateOrResolvedWork>;
}

pub enum WorkCandidateOrResolvedWork {
    Candidate(WorkCandidate),
    Resolved(ResolvedWork),
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DuplicateResolver {
    fn resolve(&self, items: Vec<ResolvedWork>) -> Vec<ResolvedWork>;
}
