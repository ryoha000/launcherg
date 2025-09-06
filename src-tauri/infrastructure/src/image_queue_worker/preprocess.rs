use domain::save_image_queue::ImagePreprocess;

pub fn run_preprocess(src_path: &str, dst_path: &str, preprocess: ImagePreprocess) -> anyhow::Result<()> {
    match preprocess {
        ImagePreprocess::ResizeAndCropSquare256 => {
            crate::icon::process_square_icon(src_path, dst_path, 256)
        }
        ImagePreprocess::ResizeForWidth400 => {
            crate::thumbnail::resize_image(src_path, dst_path, 400)
        }
        ImagePreprocess::None => {
            std::fs::copy(src_path, dst_path).map(|_| ()).map_err(|e| anyhow::anyhow!(e))
        }
    }
}


