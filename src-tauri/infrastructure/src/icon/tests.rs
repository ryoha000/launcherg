use super::*;
use std::io::Write as _;
use image::{io::Reader as ImageReader, ColorType, ImageEncoder};

#[test]
#[ignore]
fn 期待出力を生成する() {
	let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("src")
		.join("icon");
	let src_path = base.join("test_src.jpg");
	let dst_path = base.join("test_dest.jpg");
	assert!(src_path.exists(), "test_src.jpg not found: {}", src_path.display());
	crate::infrastructure::icon::process_square_icon(&src_path.to_string_lossy(), &dst_path.to_string_lossy(), 256).unwrap();
	assert!(dst_path.exists(), "failed to write: {}", dst_path.display());
}

#[test]
fn 正方形に縮小して中央切り抜きされる() {
	// 400x200 の画像を生成
	let mut img = image::RgbaImage::new(400, 200);
	for (x, y, pixel) in img.enumerate_pixels_mut() {
		let v = ((x + y) % 255) as u8;
		*pixel = image::Rgba([v, 0, 255 - v, 255]);
	}

	let tmp_dir = std::env::temp_dir();
	let src_path = tmp_dir.join("test_icon_src.png");
	let dst_path = tmp_dir.join("test_icon_dst.png");
	{
		let mut buf = Vec::new();
		image::codecs::png::PngEncoder::new(&mut buf)
			.write_image(&img, 400, 200, ColorType::Rgba8)
			.unwrap();
		let mut f = std::fs::File::create(&src_path).unwrap();
		f.write_all(&buf).unwrap();
	}

	crate::infrastructure::icon::process_square_icon(&src_path.to_string_lossy(), &dst_path.to_string_lossy(), 256).unwrap();

	let out = ImageReader::open(&dst_path).unwrap().with_guessed_format().unwrap().decode().unwrap();
	assert_eq!(out.width(), 256);
	assert_eq!(out.height(), 256);
	let _ = std::fs::remove_file(src_path);
	let _ = std::fs::remove_file(dst_path);
}

#[test]
fn 既知入力から同一画像が生成される() {
	// 入力と期待出力（リポジトリ内に生成済み）
	let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("src")
		.join("icon");
	let src_path = base.join("test_src.jpg");
	let expected_path = base.join("test_dest.jpg");
	assert!(src_path.exists(), "test_src.jpg not found: {}", src_path.display());
	assert!(expected_path.exists(), "test_dest.jpg not found: {}", expected_path.display());

	// 一時出力を生成
	let tmp_dir = std::env::temp_dir();
	let dst_path = tmp_dir.join("test_dest_gen.jpg");
	crate::infrastructure::icon::process_square_icon(&src_path.to_string_lossy(), &dst_path.to_string_lossy(), 256).unwrap();

	// 期待と実際をデコードして画素等価で比較
	let expected = ImageReader::open(&expected_path).unwrap().with_guessed_format().unwrap().decode().unwrap().to_rgba8();
	let got = ImageReader::open(&dst_path).unwrap().with_guessed_format().unwrap().decode().unwrap().to_rgba8();
	assert_eq!(expected.width(), got.width());
	assert_eq!(expected.height(), got.height());
	assert_eq!(expected.as_raw(), got.as_raw(), "pixel data differs");

	let _ = std::fs::remove_file(dst_path);
}


