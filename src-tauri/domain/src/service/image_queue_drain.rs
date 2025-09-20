#[trait_variant::make(Send)]
#[mockall::automock]
pub trait ImageQueueDrainService {
    async fn drain_until_empty(&self) -> anyhow::Result<()>;
}
