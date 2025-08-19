#[cfg(test)]
mod tests;

use std::{path::Path, fs};

use async_trait::async_trait;
use tauri::AppHandle;
use std::sync::Arc;

use crate::domain::{collection::CollectionElement, Id, icon::IconService};
use crate::domain::file::{save_icon_to_png as domain_save_icon_to_png, get_icon_path as domain_get_icon_path};
use crate::infrastructure::thumbnail as thumb;
use anyhow::Context as _;
use image::{io::Reader as ImageReader, ColorType, ImageEncoder};
use fast_image_resize as fr;
use std::io::BufWriter;

// (tests use `process_square_icon` exposed at module scope below)

pub fn process_square_icon(src: &str, dst: &str, target_short_px: u32) -> anyhow::Result<()> {
    let img = ImageReader::open(src).context("open image failed")?.decode().context("decode image failed")?;
    let src_w = img.width();
    let src_h = img.height();

    let dst_w;
    let dst_h;
    if src_w <= src_h {
        dst_w = target_short_px;
        dst_h = ((src_h as f32 / src_w as f32) * target_short_px as f32).round() as u32;
    } else {
        dst_h = target_short_px;
        dst_w = ((src_w as f32 / src_h as f32) * target_short_px as f32).round() as u32;
    }

    let width = std::num::NonZeroU32::new(src_w).ok_or_else(|| anyhow::anyhow!("invalid width"))?;
    let height = std::num::NonZeroU32::new(src_h).ok_or_else(|| anyhow::anyhow!("invalid height"))?;
    let mut src_image = fr::Image::from_vec_u8(width, height, img.to_rgba8().into_raw(), fr::PixelType::U8x4)?;

    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut())?;

    let dst_width = std::num::NonZeroU32::new(dst_w).ok_or_else(|| anyhow::anyhow!("invalid dst width"))?;
    let dst_height = std::num::NonZeroU32::new(dst_h).ok_or_else(|| anyhow::anyhow!("invalid dst height"))?;
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());
    let mut dst_view = dst_image.view_mut();
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Box));
    resizer.resize(&src_image.view(), &mut dst_view)?;
    alpha_mul_div.divide_alpha_inplace(&mut dst_view)?;

    let resized_rgba = dst_image.buffer().to_vec();
    let mut resized = image::RgbaImage::from_raw(dst_w, dst_h, resized_rgba)
        .ok_or_else(|| anyhow::anyhow!("failed to construct rgba image"))?;

    let x = ((dst_w.saturating_sub(target_short_px)) / 2).min(dst_w);
    let y = ((dst_h.saturating_sub(target_short_px)) / 2).min(dst_h);
    let cropped = image::imageops::crop_imm(&mut resized, x, y, target_short_px, target_short_px).to_image();

    let mut result_buf = BufWriter::new(fs::File::create(&dst)?);
    image::codecs::png::PngEncoder::new(&mut result_buf).write_image(
        &cropped,
        target_short_px,
        target_short_px,
        ColorType::Rgba8,
    )?;

    Ok(())
}

enum Backend {
	Tauri(Arc<AppHandle>),
	Host { root_dir: String },
}

pub struct IconServiceImpl {
	backend: Backend,
}

impl IconServiceImpl {
	pub fn new_from_app_handle(handle: Arc<AppHandle>) -> Self { Self { backend: Backend::Tauri(handle) } }
	pub fn new_from_root_path(root_dir: String) -> Self { Self { backend: Backend::Host { root_dir } } }

	pub(crate) fn build_icon_path_host(root_dir: &str, id: &Id<CollectionElement>) -> anyhow::Result<String> {
		let dir = Path::new(root_dir).join("game-icons");
		fs::create_dir_all(&dir).ok();
		Ok(dir.join(format!("{}.png", id.value)).to_string_lossy().to_string())
	}

	pub(crate) fn write_default_icon(save_path: &str) -> anyhow::Result<()> {
		let bytes = include_bytes!("..\\..\\..\\icons\\notfound.png");
		let mut file = std::fs::File::create(save_path)?;
		use std::io::Write as _;
		file.write_all(bytes)?;
		Ok(())
	}

	const ICON_TARGET_SHORT_PX: u32 = 256;
}

#[async_trait]
impl IconService for IconServiceImpl {
	async fn save_icon_from_path(&self, id: &Id<CollectionElement>, source_path: &str) -> anyhow::Result<()> {
		match &self.backend {
			Backend::Tauri(handle) => {
				let _ = domain_save_icon_to_png(handle, source_path, id)?.await??;
				Ok(())
			}
			Backend::Host { root_dir } => {
				let save_path = Self::build_icon_path_host(root_dir, id)?;
				// Host では PNG のみ受け付け、それ以外はフォールバック
				if source_path.to_lowercase().ends_with("png") {
					match std::fs::copy(source_path, &save_path) {
						Ok(_) => Ok(()),
						Err(e) => {
							log::warn!("copy png failed: {}", e);
							Self::write_default_icon(&save_path)
						}
					}
				} else {
					// fallback: デフォルト
					Self::write_default_icon(&save_path)
				}
			}
		}
	}

	async fn save_icon_from_url(&self, id: &Id<CollectionElement>, url: &str) -> anyhow::Result<()> {
		if url.is_empty() {
			return Ok(());
		}
		match &self.backend {
			Backend::Tauri(handle) => {
				let save_path = domain_get_icon_path(handle, id);
				// 既に存在すればスキップ
				if Path::new(&save_path).exists() { return Ok(()); }
				let dir = Path::new(&save_path).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| Path::new(".").to_path_buf());
				fs::create_dir_all(&dir).ok();
				let filename = url::Url::parse(url)
					.ok()
					.and_then(|u| u.path_segments().and_then(|s| s.last()).map(|s| s.to_string()))
					.unwrap_or_else(|| "icon".to_string());
				let orig = dir.join(format!("{}-{}", id.value, filename));
				thumb::download_to_file(url, &orig.to_string_lossy()).await?;
				match process_square_icon(&orig.to_string_lossy(), &save_path, Self::ICON_TARGET_SHORT_PX) {
					Ok(_) => Ok(()),
					Err(e) => {
						log::warn!("icon process failed: {}", e);
						Self::write_default_icon(&save_path)
					}
				}
			}
			Backend::Host { root_dir } => {
				let save_path = Self::build_icon_path_host(root_dir, id)?;
				if Path::new(&save_path).exists() { return Ok(()); }
				let dir = Path::new(root_dir).join("game-icons");
				fs::create_dir_all(&dir).ok();
				let filename = url::Url::parse(url)
					.ok()
					.and_then(|u| u.path_segments().and_then(|s| s.last()).map(|s| s.to_string()))
					.unwrap_or_else(|| "icon".to_string());
				let orig = dir.join(format!("{}-{}", id.value, filename));
				thumb::download_to_file(url, &orig.to_string_lossy()).await?;
				match process_square_icon(&orig.to_string_lossy(), &save_path, Self::ICON_TARGET_SHORT_PX) {
					Ok(_) => Ok(()),
					Err(e) => {
						log::warn!("icon process failed: {}", e);
						Self::write_default_icon(&save_path)
					}
				}
			}
		}
	}

	async fn save_default_icon(&self, id: &Id<CollectionElement>) -> anyhow::Result<()> {
		match &self.backend {
			Backend::Tauri(handle) => {
				let save_path = domain_get_icon_path(handle, id);
				Self::write_default_icon(&save_path)
			}
			Backend::Host { root_dir } => {
				let save_path = Self::build_icon_path_host(root_dir, id)?;
				Self::write_default_icon(&save_path)
			}
		}
	}
}
