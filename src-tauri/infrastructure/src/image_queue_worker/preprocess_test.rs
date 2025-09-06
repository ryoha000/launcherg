use std::path::Path;

use super::preprocess::run_preprocess;
use domain::save_image_queue::ImagePreprocess;

fn write_small_png(path: &str, w: u32, h: u32) {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba([0u8, 0u8, 0u8, 255u8]));
    std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
    img.save(path).unwrap();
}

#[test]
fn square256_正方形に縮小して中央切り抜きされる() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.png");
    let dst = tmp.path().join("dst.png");
    write_small_png(&src.to_string_lossy(), 800, 600);
    run_preprocess(&src.to_string_lossy(), &dst.to_string_lossy(), ImagePreprocess::ResizeAndCropSquare256).unwrap();
    assert!(Path::new(&dst).exists());
    let (w, h) = image::image_dimensions(&dst).unwrap();
    assert_eq!((w, h), (256, 256));
}

#[test]
fn resize_width400_幅400にリサイズされる() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.png");
    let dst = tmp.path().join("dst.png");
    write_small_png(&src.to_string_lossy(), 1000, 500);
    run_preprocess(&src.to_string_lossy(), &dst.to_string_lossy(), ImagePreprocess::ResizeForWidth400).unwrap();
    assert!(Path::new(&dst).exists());
    let (w, _h) = image::image_dimensions(&dst).unwrap();
    assert_eq!(w, 400);
}

#[test]
fn copy_as_is_内容がコピーされる() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.png");
    let dst = tmp.path().join("dst.png");
    write_small_png(&src.to_string_lossy(), 16, 16);
    run_preprocess(&src.to_string_lossy(), &dst.to_string_lossy(), ImagePreprocess::None).unwrap();
    assert!(Path::new(&dst).exists());
    assert_eq!(std::fs::read(&src).unwrap(), std::fs::read(&dst).unwrap());
}


