pub mod icon;
pub mod image_queue_worker;
pub mod native_messaging;
#[cfg(test)]
pub mod native_messaging_mock;
pub mod pubsubimpl;
pub mod repositoryimpl;
#[cfg(any(test, feature = "mocks"))]
pub mod repositorymock;
pub mod thumbnail;
pub mod windowsimpl;
#[cfg(any(test, feature = "mocks"))]
pub mod windowsmock;
pub use repositoryimpl::repository::{Repositories, RepositoriesExt};
// 互換レイヤ: `crate::domain` 参照を解決（infrastructure 内部が `crate::domain` を参照するため）
pub mod domain { pub use ::domain::*; pub use ::domain as domain; }
// 互換レイヤ: `crate::infrastructure::...` 参照を自身へ向ける
pub mod infrastructure { pub use super::*; }

