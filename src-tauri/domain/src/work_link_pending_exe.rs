use crate::{works::Work, Id, StrId};
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct WorkLinkPendingExe {
    pub id: Id<WorkLinkPendingExe>,
    pub work_id: StrId<Work>,
    pub exe_path: String,
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkLinkPendingExeRepository {
    /// すべての pending exe レコードを取得
    async fn list_all(&mut self) -> Result<Vec<WorkLinkPendingExe>>;
    /// レコードを削除
    async fn delete(&mut self, id: Id<WorkLinkPendingExe>) -> Result<()>;
}

