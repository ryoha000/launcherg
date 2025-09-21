use std::path::PathBuf;

use crate::{scan::CandidateKind, works::Work, Id};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkLinkTask {
    pub work_id: Id<Work>,
    pub kind: CandidateKind,
    pub src: PathBuf,
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkLinker {
    async fn ensure_links(&self, tasks: Vec<WorkLinkTask>) -> anyhow::Result<()>;
}
