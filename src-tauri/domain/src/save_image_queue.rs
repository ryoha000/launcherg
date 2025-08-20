use crate::Id;

#[derive(Debug, Clone, Copy)]
pub enum ImageSrcType { Url = 1, Path = 2 }

#[derive(Debug, Clone, Copy)]
pub enum ImagePreprocess { None = 0, ResizeAndCropSquare256 = 1, ResizeForWidth400 = 2 }

#[derive(Debug, Clone)]
pub struct ImageSaveQueueRow {
    pub id: Id<ImageSaveQueueRow>,
    pub src: String,
    pub src_type: ImageSrcType,
    pub dst_path: String,
    pub preprocess: ImagePreprocess,
    pub last_error: Option<String>,
}


