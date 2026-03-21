pub mod event;

pub use event::*;

#[mockall::automock]
pub trait PubSubService: Send + Sync {
    fn notify(&self, event: PubSubEvent) -> Result<(), anyhow::Error>;
}
