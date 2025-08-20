use std::path::Path;

use domain::{save_image_queue::{ImageSrcType, ImagePreprocess}};
use crate::icon::process_square_icon;
use crate::thumbnail as thumb_infra;
use domain::repository::RepositoriesExt;
use domain::native_host_log::{HostLogLevel, HostLogType};
use domain::repository::native_host_log::NativeHostLogRepository;
use domain::repository::save_image_queue::ImageSaveQueueRepository;
use domain::service::save_path_resolver::SavePathResolver;

pub struct ImageQueueWorker<R: RepositoriesExt> {
	repositories: std::sync::Arc<R>,
	resolver: std::sync::Arc<dyn SavePathResolver>,
}

impl<R: RepositoriesExt> ImageQueueWorker<R> {
	pub fn new(repositories: std::sync::Arc<R>, resolver: std::sync::Arc<dyn SavePathResolver>) -> Self { Self { repositories, resolver } }

	fn ensure_tmp_file(&self, queue_id: i32, src_url: &str) -> anyhow::Result<String> {
		Ok(self.resolver.tmp_download_path_for_queue(queue_id, src_url))
	}

	pub async fn drain_until_empty(&self) -> anyhow::Result<()> {
		let log_repo = self.repositories.host_log_repository();
		let queue_repo = self.repositories.image_queue_repository();

		let _ = log_repo.insert_log(HostLogLevel::Info, HostLogType::ImageQueueWorkerStarted, "image_queue_worker started").await;

		loop {
			let items = queue_repo.list_unfinished_oldest(50).await?;
			if items.is_empty() { break; }
			for item in items {
				let _ = log_repo.insert_log(HostLogLevel::Info, HostLogType::ImageQueueItemStarted, &format!("start id={} dst={} src={}", item.id.value, item.dst_path, item.src)).await;
				let result: anyhow::Result<()> = async {
					// 既に出力が存在するならスキップ
					if Path::new(&item.dst_path).exists() { return Ok(()); }

					// 1) src_type=urlなら一時ファイルへ保存、pathならそのまま使う
					let local_src_path: String = match item.src_type {
						ImageSrcType::Url => {
							let tmp = self.ensure_tmp_file(item.id.value, &item.src)?;
							thumb_infra::download_to_file(&item.src, &tmp).await?;
							tmp
						}
						ImageSrcType::Path => item.src.clone(),
					};

					// 2) ローカルパスに対して preprocess 実行
					match item.preprocess {
						ImagePreprocess::ResizeAndCropSquare256 => {
							process_square_icon(&local_src_path, &item.dst_path, 256)?;
						}
						ImagePreprocess::ResizeForWidth400 => {
							thumb_infra::resize_image(&local_src_path, &item.dst_path, 400)?;
						}
						ImagePreprocess::None => {
							// そのままコピー
							std::fs::copy(Path::new(&local_src_path), Path::new(&item.dst_path)).map_err(|e| anyhow::anyhow!(e))?;
						}
					}

					// URL由来の一時ファイルは削除
					if matches!(item.src_type, ImageSrcType::Url) {
						let _ = std::fs::remove_file(Path::new(&local_src_path));
					}
					Ok(())
				}.await;

				match result {
					Ok(_) => {
						let finished_id = item.id.clone();
						let finished_id_value = finished_id.value;
						let _ = queue_repo.mark_finished(finished_id).await;
						let _ = log_repo.insert_log(HostLogLevel::Info, HostLogType::ImageQueueItemSucceeded, &format!("done id={}", finished_id_value)).await;
					}
					Err(e) => {
						let failed_id = item.id.clone();
						let failed_id_value = failed_id.value;
						let _ = queue_repo.mark_failed(failed_id, &format!("{}", e)).await;
						let _ = log_repo.insert_log(HostLogLevel::Error, HostLogType::ImageQueueItemFailed, &format!("failed id={} err={}", failed_id_value, e)).await;
					}
				}
			}
		}

		let _ = log_repo.insert_log(HostLogLevel::Info, HostLogType::ImageQueueWorkerFinished, "image_queue_worker finished").await;
		Ok(())
	}
}
