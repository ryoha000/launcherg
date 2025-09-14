use std::sync::Arc;

use crate::{
    domain::{pubsub::PubSubService, repository::RepositoriesExt},
    domain::service::save_path_resolver::{DirsSavePathResolver},
    domain::windows::WindowsExt,
    infrastructure::{
        pubsubimpl::pubsub::{PubSub, PubSubExt},
        sqliterepository::{
            driver::Db,
            sqliterepository::{SqliteRepositories, SqliteRepositoryManager},
        },
        windowsimpl::windows::Windows,
        thumbnail::ThumbnailServiceImpl,
        icon::IconServiceImpl as TauriIconServiceImpl,
        native_messaging::NativeMessagingHostClientFactoryImpl,
        heuristic_metadata_extractor::HeuristicMetadataExtractor,
        heuristic_duplicate_resolver::HeuristicDuplicateResolver,
        local_file_system::LocalFileSystem,
        image_queue_worker::ImageQueueRunnerImpl,
        image_queue_worker::handler::{ImageQueuePubSubHandler},
    },
    usecase::{
        all_game_cache::AllGameCacheUseCase, collection::CollectionUseCase,
        explored_cache::ExploredCacheUseCase, extension_manager::ExtensionManagerUseCase,
        file::FileUseCase, image::ImageUseCase, process::ProcessUseCase,
        work_omit::WorkOmitUseCase,
        host_log::HostLogUseCase,
        dmm_pack::DmmPackUseCase,
        work::WorkUseCase,
        work_pipeline::WorkPipelineUseCase,
        image_queue::ImageQueueUseCase,
    },
};
use domain::repository::manager::RepositoryManager as _;
use domain::repository::all_game_cache::AllGameCacheRepository as _;
use domain::game_matcher::{Matcher as GameMatcherImpl, GameMatcher};
use tauri::AppHandle;

pub struct Modules {
    collection_use_case: CollectionUseCase<SqliteRepositoryManager, SqliteRepositories, ThumbnailServiceImpl, Windows>,
    explored_cache_use_case: ExploredCacheUseCase<SqliteRepositoryManager, SqliteRepositories>,
    extension_manager_use_case: ExtensionManagerUseCase<PubSub, NativeMessagingHostClientFactoryImpl>,
    file_use_case: FileUseCase<Windows>,
    all_game_cache_use_case: AllGameCacheUseCase<SqliteRepositoryManager, SqliteRepositories>,
    process_use_case: ProcessUseCase<Windows>,
    image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl, Windows>,
    work_omit_use_case: WorkOmitUseCase<SqliteRepositoryManager, SqliteRepositories>,
    host_log_use_case: HostLogUseCase<SqliteRepositoryManager, SqliteRepositories>,
    dmm_pack_use_case: DmmPackUseCase<SqliteRepositoryManager, SqliteRepositories>,
    work_use_case: WorkUseCase<SqliteRepositoryManager, SqliteRepositories, Windows>,
    work_pipeline_use_case: WorkPipelineUseCase<
        SqliteRepositoryManager,
        SqliteRepositories,
        PubSub,
        LocalFileSystem,
        HeuristicMetadataExtractor,
        HeuristicDuplicateResolver,
        Windows,
    >,
    image_queue_use_case: ImageQueueUseCase<SqliteRepositoryManager, SqliteRepositories>,
    pubsub: PubSub,
    game_matcher: std::sync::Arc<dyn GameMatcher + Send + Sync>,
    image_queue_runner: std::sync::Arc<ImageQueueRunnerImpl<SqliteRepositoryManager, SqliteRepositories, Windows>>,
}
pub trait ModulesExt {
    type Repositories: RepositoriesExt;
    type Windows: WindowsExt;
    type PubSub: PubSubExt + PubSubService;

    fn collection_use_case(&self) -> &CollectionUseCase<SqliteRepositoryManager, SqliteRepositories, ThumbnailServiceImpl, Windows>;
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn extension_manager_use_case(&self) -> &ExtensionManagerUseCase<Self::PubSub, NativeMessagingHostClientFactoryImpl>;
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn file_use_case(&self) -> &FileUseCase<Windows>;
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows>;
    fn image_use_case(&self) -> &ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl, Windows>;
    fn work_omit_use_case(&self) -> &WorkOmitUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn host_log_use_case(&self) -> &HostLogUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn dmm_pack_use_case(&self) -> &DmmPackUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn work_use_case(&self) -> &WorkUseCase<SqliteRepositoryManager, SqliteRepositories, Windows>;
    fn work_pipeline_use_case(&self) -> &WorkPipelineUseCase<
        SqliteRepositoryManager,
        SqliteRepositories,
        Self::PubSub,
        LocalFileSystem,
        HeuristicMetadataExtractor,
        HeuristicDuplicateResolver,
        Windows,
    >;
    fn pubsub(&self) -> &Self::PubSub;
    fn game_matcher(&self) -> &std::sync::Arc<dyn GameMatcher + Send + Sync>;
    fn image_queue_runner(&self) -> &std::sync::Arc<ImageQueueRunnerImpl<SqliteRepositoryManager, SqliteRepositories, Windows>>;
    fn image_queue_use_case(&self) -> &ImageQueueUseCase<SqliteRepositoryManager, SqliteRepositories>;
}

impl ModulesExt for Modules {
    type Repositories = SqliteRepositories;
    type Windows = Windows;
    type PubSub = PubSub;

    fn collection_use_case(&self) -> &CollectionUseCase<SqliteRepositoryManager, SqliteRepositories, ThumbnailServiceImpl, Windows> {
        &self.collection_use_case
    }
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<SqliteRepositoryManager, SqliteRepositories> {
        &self.explored_cache_use_case
    }
    fn extension_manager_use_case(&self) -> &ExtensionManagerUseCase<Self::PubSub, NativeMessagingHostClientFactoryImpl> {
        &self.extension_manager_use_case
    }
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<SqliteRepositoryManager, SqliteRepositories> {
        &self.all_game_cache_use_case
    }
    fn file_use_case(&self) -> &FileUseCase<Windows> {
        &self.file_use_case
    }
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows> {
        &self.process_use_case
    }
    fn image_use_case(&self) -> &ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl, Windows> {
        &self.image_use_case
    }
    fn work_omit_use_case(&self) -> &WorkOmitUseCase<SqliteRepositoryManager, SqliteRepositories> { &self.work_omit_use_case }
    fn host_log_use_case(&self) -> &HostLogUseCase<SqliteRepositoryManager, SqliteRepositories> { &self.host_log_use_case }
    fn dmm_pack_use_case(&self) -> &DmmPackUseCase<SqliteRepositoryManager, SqliteRepositories> { &self.dmm_pack_use_case }
    fn work_use_case(&self) -> &WorkUseCase<SqliteRepositoryManager, SqliteRepositories, Windows> { &self.work_use_case }
    fn work_pipeline_use_case(&self) -> &WorkPipelineUseCase<
        SqliteRepositoryManager,
        SqliteRepositories,
        Self::PubSub,
        LocalFileSystem,
        HeuristicMetadataExtractor,
        HeuristicDuplicateResolver,
        Windows,
    > { &self.work_pipeline_use_case }
    fn pubsub(&self) -> &Self::PubSub {
        &self.pubsub
    }
    fn game_matcher(&self) -> &std::sync::Arc<dyn GameMatcher + Send + Sync> { &self.game_matcher }
    fn image_queue_runner(&self) -> &std::sync::Arc<ImageQueueRunnerImpl<SqliteRepositoryManager, SqliteRepositories, Windows>> { &self.image_queue_runner }
    fn image_queue_use_case(&self) -> &ImageQueueUseCase<SqliteRepositoryManager, SqliteRepositories> { &self.image_queue_use_case }
}

impl Modules {
    pub async fn new(db: Db, handle: &AppHandle) -> Self {
        let repo_manager = Arc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let windows = Arc::new(Windows::new());
        let pubsub = PubSub::new(Arc::new(handle.clone()));
        let resolver = Arc::new(DirsSavePathResolver::default());

        let thumbs = Arc::new(ThumbnailServiceImpl::new(resolver.clone()));
        let icons = TauriIconServiceImpl::new_from_app_handle(Arc::new(handle.clone()));

        let collection_use_case = CollectionUseCase::new(repo_manager.clone(), resolver.clone(), thumbs.clone(), windows.clone());
        let explored_cache_use_case = ExploredCacheUseCase::new(repo_manager.clone());
        let extension_manager_use_case = ExtensionManagerUseCase::new(pubsub.clone(), Arc::new(NativeMessagingHostClientFactoryImpl));

        let file_use_case: FileUseCase<Windows> = FileUseCase::new(resolver.clone(), windows.clone());

        let process_use_case: ProcessUseCase<Windows> = ProcessUseCase::new(windows.clone());

        let image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl, Windows> = ImageUseCase::new(thumbs.clone(), Arc::new(icons), resolver.clone(), windows.clone());
        let work_omit_use_case: WorkOmitUseCase<SqliteRepositoryManager, SqliteRepositories> = WorkOmitUseCase::new(repo_manager.clone());
        let host_log_use_case: HostLogUseCase<SqliteRepositoryManager, SqliteRepositories> = HostLogUseCase::new(repo_manager.clone());
        let dmm_pack_use_case: DmmPackUseCase<SqliteRepositoryManager, SqliteRepositories> = DmmPackUseCase::new(repo_manager.clone());
        let work_use_case: WorkUseCase<SqliteRepositoryManager, SqliteRepositories, Windows> = WorkUseCase::new(repo_manager.clone(), windows.clone());
        let image_queue_use_case: ImageQueueUseCase<SqliteRepositoryManager, SqliteRepositories> = ImageQueueUseCase::new(repo_manager.clone());

        // GameMatcher 構築
        let initial_cache = repo_manager
            .run(|repos| Box::pin(async move { repos.all_game_cache().get_all().await }))
            .await
            .unwrap_or_else(|_| vec![]);
        let game_matcher = std::sync::Arc::new(GameMatcherImpl::with_default_config(initial_cache));
        // AllGameCacheUseCase を生成（matcher を注入）
        let all_game_cache_use_case: AllGameCacheUseCase<SqliteRepositoryManager, SqliteRepositories> =
            AllGameCacheUseCase::with_matcher(repo_manager.clone(), game_matcher.clone());

        // WorkPipelineUseCase 構築
        let fs = std::sync::Arc::new(LocalFileSystem::default());
        let extractor = std::sync::Arc::new(HeuristicMetadataExtractor::new(game_matcher.clone()));
        let dedup = std::sync::Arc::new(HeuristicDuplicateResolver);
        let work_pipeline_use_case: WorkPipelineUseCase<
            SqliteRepositoryManager,
            SqliteRepositories,
            PubSub,
            LocalFileSystem,
            HeuristicMetadataExtractor,
            HeuristicDuplicateResolver,
            Windows,
        > = WorkPipelineUseCase::new(repo_manager.clone(), pubsub.clone(), fs, extractor, dedup, resolver.clone(), windows.clone());

        // ImageQueue のイベントハンドラ: Tauri 側は PubSub を利用
        let pubsub_handler = std::sync::Arc::new(ImageQueuePubSubHandler::new(pubsub.clone()));

        let image_queue_runner: std::sync::Arc<ImageQueueRunnerImpl<SqliteRepositoryManager, SqliteRepositories, Windows>> = std::sync::Arc::new(
            ImageQueueRunnerImpl::new_with_event_handler(repo_manager.clone(), resolver.clone(), windows.clone(), pubsub_handler)
        );

        Self {
            collection_use_case,
            explored_cache_use_case,
            extension_manager_use_case,
            all_game_cache_use_case,
            file_use_case,
            process_use_case,
            image_use_case,
            work_omit_use_case,
            host_log_use_case,
            dmm_pack_use_case,
            work_use_case,
            work_pipeline_use_case,
            pubsub,
            game_matcher,
            image_queue_runner,
            image_queue_use_case,
        }
    }
}
