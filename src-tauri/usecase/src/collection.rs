use std::{fs, sync::Arc};

use chrono::Local;
use derive_new::new;
use domain::{service::save_path_resolver::SavePathResolver, thumbnail::ThumbnailService};

use super::error::UseCaseError;
use domain::{
	collection::{
		CollectionElement, NewCollectionElement, NewCollectionElementPaths, ScannedGameElement,
	},
	Id,
};
use domain::repository::{RepositoriesExt, collection::CollectionRepository, manager::RepositoryManager};
use std::marker::PhantomData;

#[derive(new)]
pub struct CollectionUseCase<M, R, TS>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    TS: ThumbnailService,
{
	manager: Arc<M>,
	resolver: Arc<dyn SavePathResolver>,
	thumbnail_service: Arc<TS>,
	#[new(default)] _marker: PhantomData<R>,
}

impl<M, R, TS> CollectionUseCase<M, R, TS>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    TS: ThumbnailService,
{
	pub async fn upsert_collection_element(
		&self,
		source: &NewCollectionElement,
	) -> anyhow::Result<()> {
		self.manager.run(|repos| Box::pin(async move { repos.collection().upsert_collection_element(source).await })).await?;
		Ok(())
	}

	// スクレイピング情報を保存
	pub async fn upsert_collection_element_info(
		&self,
		info: &domain::collection::NewCollectionElementInfo,
	) -> anyhow::Result<()> {
		self.manager.run(|repos| Box::pin(async move { repos.collection().upsert_collection_element_info(info).await })).await?;
		Ok(())
	}

	// 関連データを含むコレクション要素を作成
	pub async fn create_collection_element(
		&self,
		element: &ScannedGameElement,
	) -> anyhow::Result<Id<CollectionElement>> {
		use domain::collection::{
			NewCollectionElement, NewCollectionElementInstall, NewCollectionElementPaths,
		};

		// 1. erogamescape_id から collection_element_id を解決/作成
		let resolved_id = {
			let egid = element.erogamescape_id;
			let name = element.gamename.clone();
			self.manager.run(|repos| Box::pin(async move {
				let mut repo = repos.collection();
				if let Some(mapped) = repo.get_collection_id_by_erogamescape_id(egid).await? {
					let new_element = NewCollectionElement::new(mapped.clone(), name.clone());
					repo.upsert_collection_element(&new_element).await?;
					Ok(mapped)
				} else {
					let id = repo.allocate_new_collection_element_id(&name).await?;
					let _ = repo.upsert_erogamescape_map(&id, egid).await;
					Ok(id)
				}
			})).await?
		};

		// 2. スクレイピング情報は初期登録時には作成しない
		// （後でregisterCollectionElementDetailsから取得される）

		// 3. パス情報を保存
		if element.exe_path.is_some() || element.lnk_path.is_some() {
			let new_paths = NewCollectionElementPaths::new(
				resolved_id.clone(),
				element.exe_path.clone(),
				element.lnk_path.clone(),
			);
			self.manager.run(|repos| Box::pin(async move { repos.collection().upsert_collection_element_paths(&new_paths).await })).await?;
		}

		// 4. インストール情報を保存
		if let Some(install_time) = element.install_at {
			let new_install = NewCollectionElementInstall::new(resolved_id.clone(), install_time);
			self.manager.run(|repos| Box::pin(async move { repos.collection().upsert_collection_element_install(&new_install).await })).await?;
		}

		Ok(resolved_id)
	}
	pub async fn upsert_collection_element_thumbnail_size(
		&self,
		id: &Id<CollectionElement>,
	) -> anyhow::Result<()> {
		let thumbnail_path = self.resolver.thumbnail_png_path(id.value);
		match image::image_dimensions(thumbnail_path) {
			Ok((width, height)) => {
				self.manager.run(|repos| Box::pin(async move { repos.collection().upsert_collection_element_thumbnail_size(id, width as i32, height as i32).await })).await?;
			}
			Err(e) => {
				eprintln!(
					"[upsert_collection_element_thumbnail_size] {}",
					e.to_string()
				);
			}
		}
		Ok(())
	}
	pub async fn concurency_upsert_collection_element_thumbnail_size(
		&self,
		ids: Vec<Id<CollectionElement>>,
	) -> anyhow::Result<()> {
		use futures::StreamExt as _;

		futures::stream::iter(ids.into_iter())
			.map(move |id| async move { self.upsert_collection_element_thumbnail_size(&id).await })
			.buffered(50)
			.for_each(|v| async move {
				match v {
					Err(e) => eprintln!(
						"[concurency_upsert_collection_element_thumbnail_size] {}",
						e.to_string()
					),
					_ => {}
				}
			})
			.await;
		Ok(())
	}

	// 関連データ付きコレクション要素リストを一括保存
	pub async fn upsert_collection_elements(
		&self,
		source: &Vec<ScannedGameElement>,
	) -> anyhow::Result<()> {
		for element in source.iter() {
			self.create_collection_element(element).await?;
		}
		Ok(())
	}

	pub async fn get_element_by_element_id(
		&self,
		id: &Id<CollectionElement>,
	) -> anyhow::Result<CollectionElement> {
		let row = self.manager.run(|repos| Box::pin(async move { repos.collection().get_element_by_element_id(id).await })).await?;
		Ok(row.ok_or(UseCaseError::CollectionElementIsNotFound)?)
	}

	pub async fn update_collection_element_icon(
		&self,
		id: &Id<CollectionElement>,
		path: String,
	) -> anyhow::Result<()> {
		let save_icon_path = self.resolver.icon_png_path(id.value);
		fs::copy(path, save_icon_path)?;
		Ok(())
	}

	pub async fn save_element_icon(
		&self,
		id: &Id<CollectionElement>,
	) -> anyhow::Result<()> {
		let paths = self.manager.run(|repos| Box::pin(async move { repos.collection().get_element_paths_by_element_id(id).await })).await?;

		let _icon_path = if let Some(paths) = paths {
			if let Some(lnk_path) = paths.lnk_path {
				// lnkファイルからメタデータを取得してアイコンパスを決定
				use domain::file::get_lnk_metadatas;
				let metadatas = get_lnk_metadatas(vec![lnk_path.as_str()])?;
				let metadata = metadatas
					.get(lnk_path.as_str())
					.ok_or(anyhow::anyhow!("metadata cannot get"))?;
				let dst = self.resolver.icon_png_path(id.value);
				if metadata.icon.to_lowercase().ends_with("ico") {
					log::info!("icon is ico");
					domain::file::save_ico_to_png(&metadata.icon, &dst)?.await??;
				} else {
					// exe抽出はAppHandleが必要なため、ここではコピーにフォールバック
					let _ = std::fs::copy(&metadata.path, &dst);
				}
				dst
			} else if let Some(exe_path) = paths.exe_path {
				let dst = self.resolver.icon_png_path(id.value);
				let _ = std::fs::copy(&exe_path, &dst);
				dst
			} else {
				eprintln!("lnk_path and exe_path are None");
				return Ok(());
			}
		} else {
			eprintln!("No paths found for element {}", id.value);
			return Ok(());
		};
		Ok(())
	}

	pub async fn save_element_thumbnail(
		&self,
		id: &Id<CollectionElement>,
		src_url: String,
	) -> anyhow::Result<()> {
		self.thumbnail_service.save_thumbnail(id, &src_url).await
	}

	pub async fn concurency_save_thumbnails(
		&self,
		args: Vec<(Id<CollectionElement>, String)>,
	) -> anyhow::Result<()> {
		use futures::StreamExt as _;
		futures::stream::iter(args.into_iter())
			.map(move |(id, url)| async move { self.thumbnail_service.save_thumbnail(&id, &url).await })
			.buffered(50)
			.for_each(|res| async move {
				if let Err(e) = res {
					eprintln!("[concurency_save_thumbnails] {}", e);
				}
			})
			.await;
		Ok(())
	}

	pub async fn delete_collection_element_by_id(
		&self,
		id: &Id<CollectionElement>,
	) -> anyhow::Result<()> {
		let existed = self.manager.run(|repos| Box::pin(async move { repos.collection().get_element_by_element_id(id).await })).await?;
		if existed.is_none() {
			return Err(UseCaseError::CollectionElementIsNotFound.into());
		}
		self.manager.run(|repos| Box::pin(async move { repos.collection().delete_collection_element(id).await })).await
	}

	pub async fn get_not_registered_detail_element_ids(
		&self,
	) -> anyhow::Result<Vec<Id<CollectionElement>>> {
		self.manager.run(|repos| Box::pin(async move { repos.collection().get_not_registered_info_element_ids().await })).await
	}

	pub async fn update_element_last_play_at(
		&self,
		id: &Id<CollectionElement>,
	) -> anyhow::Result<()> {
		self.manager.run(|repos| Box::pin(async move { repos.collection().update_element_last_play_at_by_id(id, Local::now()).await })).await?;
		Ok(())
	}
	pub async fn update_element_like_at(
		&self,
		id: &Id<CollectionElement>,
		is_like: bool,
	) -> anyhow::Result<()> {
		self.manager.run(|repos| Box::pin(async move { repos.collection().update_element_like_at_by_id(id, is_like.then_some(Local::now())).await })).await?;
		Ok(())
	}
	pub async fn get_all_elements(
		&self,
	) -> anyhow::Result<Vec<CollectionElement>> {
		let null_size_ids = self.manager.run(|repos| Box::pin(async move { repos.collection().get_null_thumbnail_size_element_ids().await })).await?;
		self.concurency_upsert_collection_element_thumbnail_size(null_size_ids)
			.await?;

		self.manager.run(|repos| Box::pin(async move { repos.collection().get_all_elements().await })).await
	}

	pub async fn link_installed_game(
		&self,
		collection_element_id: Id<CollectionElement>,
		exe_path: String,
	) -> anyhow::Result<()> {
		let paths = NewCollectionElementPaths::new(
			collection_element_id.clone(),
			Some(exe_path),
			None, // lnk_path
		);

		self.manager.run(|repos| Box::pin(async move { repos.collection().upsert_collection_element_paths(&paths).await })).await?;

		Ok(())
	}

	// EGS ID から collection_element_id 群を解決
	pub async fn get_collection_ids_by_erogamescape_ids(
		&self,
		erogamescape_ids: Vec<i32>,
	) -> anyhow::Result<Vec<Id<CollectionElement>>> {
		self.manager.run(|repos| Box::pin(async move {
			let mut repo = repos.collection();
			let mut ids = Vec::new();
			for egs_id in erogamescape_ids {
				if let Some(id) = repo.get_collection_id_by_erogamescape_id(egs_id).await? {
					ids.push(id);
				}
			}
			Ok(ids)
		})).await
	}

	// collection_element_id -> erogamescape_id（単発）
	pub async fn get_erogamescape_id_by_collection_id(
		&self,
		id: &Id<CollectionElement>,
	) -> anyhow::Result<Option<i32>> {
		self.manager.run(|repos| Box::pin(async move { repos.collection().get_erogamescape_id_by_collection_id(id).await })).await
	}
}
