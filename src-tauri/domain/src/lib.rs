#[path = "../../src/domain/mod.rs"]
pub mod domain;
pub use domain::*;
// 互換レイヤ: `crate::domain::domain::...` のような二重参照を避けるためエイリアス
pub use domain as domain_mod;

