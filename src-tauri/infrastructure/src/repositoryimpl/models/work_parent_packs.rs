#[derive(sqlx::FromRow, Clone)]
pub struct WorkParentPackTable {
    pub id: i64,
    pub work_id: i64,
    pub parent_pack_work_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl TryFrom<crate::repositoryimpl::models::work_parent_packs::WorkParentPackTable> for domain::work_parent_pack::WorkParentPack {
    type Error = anyhow::Error;
    fn try_from(v: crate::repositoryimpl::models::work_parent_packs::WorkParentPackTable) -> Result<Self, Self::Error> {
        Ok(domain::work_parent_pack::WorkParentPack {
            id: domain::Id::new(v.id as i32),
            work_id: domain::Id::new(v.work_id as i32),
            parent_pack_work_id: domain::Id::new(v.parent_pack_work_id as i32),
            created_at: v.created_at,
            updated_at: v.updated_at,
        })
    }
}
