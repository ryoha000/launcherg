use domain::service::save_path_resolver::SavePathResolver;

pub async fn resolve_to_tmp(resolver: &dyn SavePathResolver, src_url: &str) -> anyhow::Result<String> {
    let ext = std::path::Path::new(src_url)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let tmp = resolver.tmp_unique_path_with_ext(ext);
    crate::thumbnail::download_to_file(src_url, &tmp).await?;
    Ok(tmp)
}


