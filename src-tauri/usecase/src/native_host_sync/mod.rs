//! ネイティブホスト同期ユースケース: ストア(DMM/DLsite)のゲーム情報をコレクションへ同期する。
//! - 既存マッピングがあればスキップして冪等性を保つ
//! - EGS 情報があれば名称/詳細も upsert し、EGS マップを作成/更新する

use derive_new::new;
use domain::repository::{manager::RepositoryManager, RepositoriesExt};
use domain::service::work_registration::WorkRegistrationService;
use std::marker::PhantomData;
use std::sync::Arc;

mod dlsite;
mod dmm;
pub mod downloads;

/// 拡張から渡された image_url/thumbnail_url を保存に適したサムネイルURLへ正規化する
/// - DLsite: /resize/images2/.../_img_main_300x300.jpg → /modpub/images2/.../_img_main.jpg
/// - DMM:    ...ps.jpg → ...pl.jpg
pub(crate) fn normalize_thumbnail_url(src_url: &str) -> String {
    let mut url = src_url.to_string();
    if url.contains("img.dlsite.jp") {
        url = url.replace("/resize/images2/", "/modpub/images2/");
        if let Some(last_slash) = url.rfind('/') {
            let (base, file) = url.split_at(last_slash + 1);
            if let Some(main_pos) = file.find("_img_main_") {
                let (prefix, _) = file.split_at(main_pos);
                url = format!("{}{}_img_main.jpg", base, prefix);
            }
        }
    } else if url.contains("pics.dmm.co.jp") {
        if url.ends_with("ps.jpg") {
            url = url.trim_end_matches("ps.jpg").to_string() + "pl.jpg";
        }
    }
    url
}

// 旧 build_icon_dst_path / build_thumbnail_resized_dst_path は SavePathResolver に移管

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DmmPackKey {
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
}

#[derive(Clone, Debug)]
/// DMM 由来のゲーム同期パラメータ。キーは `(store_id, category, subcategory)`。
/// - `egs` が `Some` の場合、EGS 情報をコレクションへ反映し、EGS マップも作成/更新する。
/// - `egs` が `None` の場合、与えられた `gamename` を用いて要素IDを採番し、マッピングのみ作成する。
pub struct DmmSyncGameParam {
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
    pub gamename: String,
    pub egs: Option<EgsInfo>,
    pub image_url: String,
    pub parent_pack: Option<DmmPackKey>,
}

#[derive(Clone, Debug)]
/// DLsite 由来のゲーム同期パラメータ。キーは `(store_id, category)`。
/// 取り扱いは DMM と同様で、`egs` の有無に応じてコレクション側の更新内容が変わる。
pub struct DlsiteSyncGameParam {
    pub store_id: String,
    pub category: String,
    pub gamename: String,
    pub egs: Option<EgsInfo>,
    pub image_url: String,
}

#[derive(Clone, Debug)]
/// ErogameScape(EGS) 由来のメタ情報。
/// コレクション要素の名称・詳細情報に反映され、`erogamescape_id` は EGS マップのキーとなる。
pub struct EgsInfo {
    pub erogamescape_id: i32,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

// DMM 向けの計画決定ロジックは dmm.rs へ移動

#[derive(new)]
pub struct NativeHostSyncUseCase<M, R, RS>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    RS: WorkRegistrationService + Send + Sync + 'static,
{
    #[allow(dead_code)]
    manager: Arc<M>,
    registrar: Arc<RS>,
    #[new(default)]
    _marker: PhantomData<R>,
}
