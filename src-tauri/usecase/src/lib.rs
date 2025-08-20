pub mod all_game_cache;
mod all_game_cache_test;
pub mod collection;
mod collection_test;
pub mod error;
pub mod explored_cache;
mod explored_cache_test;
pub mod extension_installer;
pub mod extension_manager;
mod extension_manager_test;
pub mod image;
pub mod file;
mod file_test;
pub mod game_identifier;
pub mod models;
pub mod process;
pub mod native_host_sync;
#[cfg(test)]
mod repositorymock;
#[cfg(test)]
mod windowsmock;
#[cfg(test)]
mod native_messaging_mock;
// 再エクスポートは不要（各モジュールを直接公開）
// 互換レイヤ: monorepo 時代の `crate::domain` / `crate::infrastructure` を解決
pub mod domain { pub use ::domain::*; }
pub mod infrastructure { pub use ::infrastructure::*; }
// 互換レイヤ: crate::usecase::... を許容（既存テスト互換）
pub mod usecase { pub use super::*; }

