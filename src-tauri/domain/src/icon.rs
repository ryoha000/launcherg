use crate::{works::Work, Id};

#[trait_variant::make(Send)]
pub trait IconService {
    async fn save_icon_from_path(
        &self,
        id: &Id<Work>,
        source_path: &str,
    ) -> anyhow::Result<()>;

    /// URL からアイコン画像を取得し、短辺を既定長に合わせてから中央を正方形に切り抜いて保存する
    async fn save_icon_from_url(&self, id: &Id<Work>, url: &str)
        -> anyhow::Result<()>;

    async fn save_default_icon(&self, id: &Id<Work>) -> anyhow::Result<()>;
}
