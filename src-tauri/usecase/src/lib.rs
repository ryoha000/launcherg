#[path = "../../src/usecase/mod.rs"]
mod usecase_impl;
pub use usecase_impl::*;
// 互換レイヤ: monorepo 時代の `crate::domain` / `crate::infrastructure` を解決
pub mod domain { pub use ::domain::*; }
pub mod infrastructure { pub use ::infrastructure::*; }

