use crate::domain::save_image_queue::{
    ImagePreprocess as DomainPreprocess, ImageSaveQueueRow as DomainRow,
    ImageSrcType as DomainSrcType,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSaveQueueRowVm {
    pub id: i32,
    pub src: String,
    pub src_type: i32,
    pub dst_path: String,
    pub preprocess: i32,
    pub last_error: Option<String>,
}

impl From<DomainRow> for ImageSaveQueueRowVm {
    fn from(v: DomainRow) -> Self {
        Self {
            id: v.id.value,
            src: v.src,
            src_type: match v.src_type {
                DomainSrcType::Url => 1,
                DomainSrcType::Path => 2,
                DomainSrcType::Exe => 3,
                DomainSrcType::Shortcut => 4,
            },
            dst_path: v.dst_path,
            preprocess: match v.preprocess {
                DomainPreprocess::None => 0,
                DomainPreprocess::ResizeAndCropSquare256 => 1,
                DomainPreprocess::ResizeForWidth400 => 2,
            },
            last_error: v.last_error,
        }
    }
}
