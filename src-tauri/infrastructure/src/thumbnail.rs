use std::{fs, io::{BufWriter, Write}, num::NonZeroU32, path::Path};

use anyhow::Context as _;
use fast_image_resize as fr;
use image::{io::Reader as ImageReader, ColorType, ImageEncoder};

use domain::{collection::CollectionElement, Id, thumbnail::ThumbnailService};
use domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};

pub fn build_thumbnail_paths(id: &Id<CollectionElement>, src_url: &str) -> anyhow::Result<(String, String)> {
    // 互換: 既存の呼び出し用（今後は SavePathResolver へ移行）
    let resolver = DirsSavePathResolver::default();
    let resized = resolver.thumbnail_png_path(id.value);
    let orig = resolver.tmp_download_path_for_id(id.value, src_url);
    Ok((orig, resized))
}

pub async fn download_to_file(url: &str, path: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let res = client.get(url).send().await.context("download failed")?;
    let bytes = res.bytes().await.context("read body failed")?;
    let mut file = std::fs::File::create(path).context("create file failed")?;
    file.write_all(&bytes).context("write file failed")?;
    Ok(())
}

pub fn resize_image(src: &str, dst: &str, dst_width_px: u32) -> anyhow::Result<()> {
    let img = ImageReader::open(src).context("open image failed")?.decode().context("decode image failed")?;
    let width = NonZeroU32::new(img.width()).ok_or_else(|| anyhow::anyhow!("invalid width"))?;
    let height = NonZeroU32::new(img.height()).ok_or_else(|| anyhow::anyhow!("invalid height"))?;
    let mut src_image = fr::Image::from_vec_u8(width, height, img.to_rgba8().into_raw(), fr::PixelType::U8x4)?;

    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut())?;

    let dst_width = NonZeroU32::new(dst_width_px).ok_or_else(|| anyhow::anyhow!("invalid dst width"))?;
    let dst_height = NonZeroU32::new((height.get() as f32 / width.get() as f32 * dst_width_px as f32) as u32)
        .ok_or_else(|| anyhow::anyhow!("invalid dst height"))?;
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());
    let mut dst_view = dst_image.view_mut();
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Box));
    resizer.resize(&src_image.view(), &mut dst_view)?;
    alpha_mul_div.divide_alpha_inplace(&mut dst_view)?;

    let mut result_buf = BufWriter::new(fs::File::create(&dst)?);
    image::codecs::png::PngEncoder::new(&mut result_buf).write_image(
        dst_image.buffer(),
        dst_width.get(),
        dst_height.get(),
        ColorType::Rgba8,
    )?;

    Ok(())
}

pub async fn save_thumbnail(id: &Id<CollectionElement>, url: &str, width: u32) -> anyhow::Result<()> {
    if url.is_empty() {
        return Ok(());
    }
    let (_orig, resized) = build_thumbnail_paths(id, url)?;
    if Path::new(&resized).exists() {
        return Ok(());
    }
    let (orig, resized) = build_thumbnail_paths(id, url)?;
    download_to_file(url, &orig).await?;
    resize_image(&orig, &resized, width)?;
    Ok(())
}

pub struct ThumbnailServiceImpl {
    resolver: std::sync::Arc<dyn SavePathResolver>,
}

impl ThumbnailServiceImpl {
    pub fn new(resolver: std::sync::Arc<dyn SavePathResolver>) -> Self { Self { resolver } }
}

impl ThumbnailService for ThumbnailServiceImpl {
    async fn save_thumbnail(&self, id: &Id<CollectionElement>, url: &str) -> anyhow::Result<()> {
        save_thumbnail(id, url, 400).await
    }

    async fn get_thumbnail_size(&self, id: &Id<CollectionElement>) -> anyhow::Result<Option<(u32, u32)>> {
        let resized = self.resolver.thumbnail_png_path(id.value);
        if !Path::new(&resized).exists() {
            return Ok(None);
        }
        match image::image_dimensions(resized) {
            Ok((w, h)) => Ok(Some((w, h))),
            Err(_) => Ok(None),
        }
    }
}
