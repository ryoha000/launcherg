use domain::service::save_path_resolver::SavePathResolver;
use crate::image_queue_worker::sidecar::{ExtractIconRunner, ExtractIconRunnerImpl};
use crate::image_queue_worker::types::{LocalSource, SourceDecision, Cleanup};

pub fn resolve(resolver: &dyn SavePathResolver, exe_path: &str) -> anyhow::Result<SourceDecision> {
    let runner = ExtractIconRunnerImpl::new();
    let dst_tmp = resolver.tmp_unique_path_with_ext("png");
    match runner.extract_icon(48, exe_path, &dst_tmp) {
        Ok(true) => Ok(SourceDecision::Use(LocalSource::new(dst_tmp, Cleanup::None))),
        _ => Ok(SourceDecision::FallbackDefaultAndSkip),
    }
}


