#[derive(sqlx::FromRow, Clone)]
pub struct WorkParentPackTable {
    pub id: i64,
    pub work_id: String,
    pub parent_pack_work_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl TryFrom<crate::sqliterepository::models::work_parent_packs::WorkParentPackTable>
    for domain::work_parent_pack::WorkParentPack
{
    type Error = anyhow::Error;
    fn try_from(
        v: crate::sqliterepository::models::work_parent_packs::WorkParentPackTable,
    ) -> Result<Self, Self::Error> {
        Ok(domain::work_parent_pack::WorkParentPack {
            id: domain::Id::new(v.id as i32),
            work_id: domain::StrId::new(v.work_id),
            parent_pack_work_id: domain::StrId::new(v.parent_pack_work_id),
            created_at: v.created_at,
            updated_at: v.updated_at,
        })
    }
}
