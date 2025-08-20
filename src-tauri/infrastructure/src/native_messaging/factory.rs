use domain::extension::NativeMessagingHostClientFactory;

use super::{client::NativeMessagingHostClientImpl, path_resolver::NativeHostPathResolver};

pub struct NativeMessagingHostClientFactoryImpl;

impl NativeMessagingHostClientFactory for NativeMessagingHostClientFactoryImpl {
    type Client = NativeMessagingHostClientImpl;

    fn create(&self) -> Result<Self::Client, Box<dyn std::error::Error + Send + Sync>> {
        let native_host_path = NativeHostPathResolver::resolve_path()?;
        Ok(NativeMessagingHostClientImpl::new(native_host_path))
    }
}


