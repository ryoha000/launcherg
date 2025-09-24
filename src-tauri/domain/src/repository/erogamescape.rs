use crate::erogamescape::NewErogamescapeInformation;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait ErogamescapeRepository {
    async fn upsert_information(&mut self, info: &NewErogamescapeInformation)
        -> anyhow::Result<()>;

    /// work_erogamescape_map に存在し、erogamescape_information に未登録の EGS ID 群
    async fn find_missing_information_ids(&mut self) -> anyhow::Result<Vec<i32>>;
}
