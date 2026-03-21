use domain::service::save_path_resolver::{DirsSavePathResolver, SavePathResolver};

use crate::interface::models::parent_dmm_pack::DmmPackKeysVm;
use crate::domain::works::WorkDetails;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkDetailsVm {
    pub id: String,
    pub title: String,
    pub dmm: Option<DmmSideVm>,
    pub dlsite: Option<DlsiteSideVm>,
    pub erogamescape_id: Option<i32>,
    pub erogamescape_information: Option<ErogamescapeInformationVm>,
    pub icon: Option<IconVm>,
    pub thumbnail: Option<ThumbnailVm>,
    pub latest_download_path: Option<LatestWorkDownloadPathVm>,
    pub original_path: Option<String>,
    pub like_at: Option<String>,
    pub install_at: Option<String>,
    pub last_play_at: Option<String>,
    pub registered_at: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmmSideVm {
    pub id: i32,
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
    pub parent_pack: Option<DmmPackKeysVm>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DlsiteSideVm {
    pub id: i32,
    pub store_id: String,
    pub category: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestWorkDownloadPathVm {
    pub id: i32,
    pub work_id: String,
    pub download_path: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErogamescapeInformationVm {
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailVm {
    pub path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconVm {
    pub path: String,
}

impl From<WorkDetails> for WorkDetailsVm {
    fn from(w: WorkDetails) -> Self {
        let resolver = DirsSavePathResolver::default();
        let icon_path = Some(resolver.icon_png_path(&w.work.id.value));
        let thumbnail_path = Some(resolver.thumbnail_png_path(&w.work.id.value));
        WorkDetailsVm {
            id: w.work.id.value.clone(),
            title: w.work.title,
            dmm: w.dmm.map(|d| DmmSideVm {
                id: d.id.value,
                store_id: d.store_id,
                category: d.category,
                subcategory: d.subcategory,
                parent_pack: d.parent_pack.map(|p| DmmPackKeysVm {
                    store_id: p.store_id,
                    category: p.category,
                    subcategory: p.subcategory,
                }),
            }),
            dlsite: w.dlsite.map(|d| DlsiteSideVm {
                id: d.id.value,
                store_id: d.store_id,
                category: d.category,
            }),
            erogamescape_id: w.erogamescape_id,
            erogamescape_information: w.erogamescape_information.map(|i| {
                ErogamescapeInformationVm {
                    gamename_ruby: i.gamename_ruby,
                    brandname: i.brandname,
                    brandname_ruby: i.brandname_ruby,
                    sellday: i.sellday,
                    is_nukige: i.is_nukige,
                }
            }),
            icon: icon_path.map(|p| IconVm {
                path: p,
            }),
            thumbnail: thumbnail_path.map(|p| ThumbnailVm {
                path: p,
                width: w.thumbnail_size.as_ref().map(|s| s.width),
                height: w.thumbnail_size.as_ref().map(|s| s.height),
            }),
            latest_download_path: w.latest_download_path.map(|p| LatestWorkDownloadPathVm {
                id: p.id.value,
                work_id: p.work_id.value.clone(),
                download_path: p.download_path,
            }),
            original_path: w.original_path.clone(),
            like_at: w
                .like
                .as_ref()
                .map(|l| l.like_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            install_at: w
                .install_at
                .as_ref()
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            last_play_at: w
                .last_play_at
                .as_ref()
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            registered_at: w
                .registered_at
                .as_ref()
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}
