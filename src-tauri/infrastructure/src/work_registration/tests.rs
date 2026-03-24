use std::sync::Arc;
use tempfile::TempDir;

use crate::sqliterepository::sqliterepository::{SqliteRepositories, SqliteRepositoryManager};
use crate::sqliterepository::tests::TestDatabase;
use crate::windowsimpl::windows::Windows;
use domain::repository::{
    all_game_cache::AllGameCacheRepository, save_image_queue::ImageSaveQueueRepository,
    work_parent_packs::WorkParentPacksRepository, works::WorkRepository, RepositoriesExt,
};
use domain::service::save_path_resolver::{DirsSavePathResolver, SavePathResolver};
use domain::service::work_registration::{
    ImageApply, ImageSource, ImageStrategy, RegisterWorkPath, UniqueWorkKey, WorkInsert,
    WorkRegistrationRequest, WorkRegistrationService,
};
use domain::works::NewWork;
use domain::windows::{
    process::MockProcessWindows, proctail::MockProcTail,
    proctail_manager::MockProcTailManagerTrait, shell_link::MockShellLink, WindowsExt,
};

use super::WorkRegistrationServiceImpl;

fn create_service(
    test_db: &TestDatabase,
) -> WorkRegistrationServiceImpl<
    SqliteRepositoryManager,
    SqliteRepositories,
    crate::windowsimpl::windows::Windows,
> {
    let windows = Arc::new(crate::windowsimpl::windows::Windows::new());
    create_service_with_windows(test_db, windows)
}

fn create_service_with_windows<W>(
    test_db: &TestDatabase,
    windows: Arc<W>,
) -> WorkRegistrationServiceImpl<SqliteRepositoryManager, SqliteRepositories, W>
where
    W: WindowsExt + Send + Sync + 'static,
{
    let manager = Arc::new(
        crate::sqliterepository::sqliterepository::SqliteRepositoryManager::new(Arc::new(
            test_db.pool.clone(),
        )),
    );
    let resolver = Arc::new(DirsSavePathResolver::default());
    WorkRegistrationServiceImpl::new(manager, resolver, windows)
}

struct TestWindows {
    process: MockProcessWindows,
    proctail: MockProcTail,
    proctail_manager: MockProcTailManagerTrait,
    shell_link: MockShellLink,
}

impl TestWindows {
    fn new(shell_link: MockShellLink) -> Self {
        Self {
            process: MockProcessWindows::new(),
            proctail: MockProcTail::new(),
            proctail_manager: MockProcTailManagerTrait::new(),
            shell_link,
        }
    }
}

impl WindowsExt for TestWindows {
    type ProcessWindows = MockProcessWindows;
    type ProcTail = MockProcTail;
    type ProcTailManager = MockProcTailManagerTrait;
    type ShellLink = MockShellLink;

    fn process(&self) -> &Self::ProcessWindows {
        &self.process
    }
    fn proctail(&self) -> &Self::ProcTail {
        &self.proctail
    }
    fn proctail_manager(&self) -> &Self::ProcTailManager {
        &self.proctail_manager
    }
    fn shell_link(&self) -> &Self::ShellLink {
        &self.shell_link
    }
}

#[tokio::test]
async fn register_画像戦略_always_は常に適用される() {
    let test_db = TestDatabase::new().await.unwrap();
    let service = create_service(&test_db);

    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(1)],
        insert: WorkInsert {
            title: "Test Work".to_string(),
            path: None,
            egs_info: None,
            icon: Some(ImageApply {
                strategy: ImageStrategy::Always,
                source: ImageSource::FromUrl("https://example.com/icon.png".to_string()),
            }),
            thumbnail: None,
            parent_pack_dmm_key: None,
        },
    }];

    let results = service.register(requests).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_new_work);

    // 画像キューに投入されていることを確認
    let repo = test_db.sqlite_repository();
    let queue_items = repo.image_queue().list(true, 10).await.unwrap();
    assert_eq!(queue_items.len(), 1);
    assert!(queue_items[0].dst_path.contains("icon"));
}

#[tokio::test]
async fn register_画像戦略_only_if_new_は新規_work_の場合のみ適用される() {
    let test_db = TestDatabase::new().await.unwrap();
    let service = create_service(&test_db);
    let repo = test_db.sqlite_repository();

    // 既存 Work を作成
    let existing_work = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "Existing".to_string(),
        })
        .await
        .unwrap()
    };

    // 既存 Work を使用するリクエスト（OnlyIfNew は適用されない）
    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(1)],
        insert: WorkInsert {
            title: "Test Work".to_string(),
            path: None,
            egs_info: None,
            icon: Some(ImageApply {
                strategy: ImageStrategy::OnlyIfNew,
                source: ImageSource::FromUrl("https://example.com/icon.png".to_string()),
            }),
            thumbnail: None,
            parent_pack_dmm_key: None,
        },
    }];

    // EGS マッピングを事前に作成
    {
        let mut r = repo.work();
        r.upsert_erogamescape_map(existing_work.clone(), 1)
            .await
            .unwrap();
    }

    let results = service.register(requests).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].work_id.value, existing_work.value);
    assert!(!results[0].is_new_work);

    // 画像キューには投入されていないことを確認
    let queue_items = repo.image_queue().list(true, 10).await.unwrap();
    assert_eq!(queue_items.len(), 0);
}

#[tokio::test]
async fn register_画像戦略_only_if_missing_はファイルが存在しない場合のみ適用される() {
    let test_db = TestDatabase::new().await.unwrap();
    let resolver = Arc::new(DirsSavePathResolver::default());
    let resolver_for_check = resolver.clone();
    let manager = Arc::new(
        crate::sqliterepository::sqliterepository::SqliteRepositoryManager::new(Arc::new(
            test_db.pool.clone(),
        )),
    );
    let windows = Arc::new(Windows::new());
    let service = WorkRegistrationServiceImpl::new(manager, resolver, windows);

    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(1)],
        insert: WorkInsert {
            title: "Test Work".to_string(),
            path: None,
            egs_info: None,
            icon: Some(ImageApply {
                strategy: ImageStrategy::OnlyIfMissing,
                source: ImageSource::FromUrl("https://example.com/icon.png".to_string()),
            }),
            thumbnail: None,
            parent_pack_dmm_key: None,
        },
    }];

    let results = service.register(requests.clone()).await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_new_work);

    // 1回目は画像キューに投入される
    let repo = test_db.sqlite_repository();
    let queue_items = repo.image_queue().list(true, 10).await.unwrap();
    assert_eq!(queue_items.len(), 1);

    // アイコンファイルを手動で作成（resolver の出力先パスに合わせる）
    let work_id = &results[0].work_id.value;
    let icon_path = resolver_for_check.icon_png_path(work_id);
    if let Some(parent) = std::path::Path::new(&icon_path).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&icon_path, b"fake icon").unwrap();

    // 2回目の登録（OnlyIfMissing なのでスキップされる）
    let queue_count_before = repo.image_queue().list(true, 10).await.unwrap().len();
    let _results2 = service.register(requests).await.unwrap();
    let queue_count_after = repo.image_queue().list(true, 10).await.unwrap().len();
    assert_eq!(queue_count_before, queue_count_after);
}

#[tokio::test]
async fn register_画像ソース_from_path_でパスから抽出される() {
    let test_db = TestDatabase::new().await.unwrap();
    let temp_dir = TempDir::new().unwrap();
    let exe_path = temp_dir.path().join("test.exe");
    std::fs::write(&exe_path, b"fake exe").unwrap();

    let resolver = Arc::new(DirsSavePathResolver::default());
    let manager = Arc::new(
        crate::sqliterepository::sqliterepository::SqliteRepositoryManager::new(Arc::new(
            test_db.pool.clone(),
        )),
    );
    let windows = Arc::new(Windows::new());
    let service = WorkRegistrationServiceImpl::new(manager, resolver, windows);

    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(1)],
        insert: WorkInsert {
            title: "Test Work".to_string(),
            path: Some(RegisterWorkPath::Exe {
                exe_path: exe_path.to_string_lossy().to_string(),
            }),
            egs_info: None,
            icon: Some(ImageApply {
                strategy: ImageStrategy::Always,
                source: ImageSource::FromPath(RegisterWorkPath::Exe {
                    exe_path: exe_path.to_string_lossy().to_string(),
                }),
            }),
            thumbnail: None,
            parent_pack_dmm_key: None,
        },
    }];

    let results = service.register(requests).await.unwrap();
    assert_eq!(results.len(), 1);

    // 画像キューに投入されていることを確認
    let repo = test_db.sqlite_repository();
    let queue_items = repo.image_queue().list(true, 10).await.unwrap();
    assert_eq!(queue_items.len(), 1);
    assert_eq!(queue_items[0].src, exe_path.to_string_lossy().to_string());
}

#[tokio::test]
async fn register_exe登録時はlnkのworking_dirがexeのあるディレクトリになる() {
    let test_db = TestDatabase::new().await.unwrap();
    let temp_dir = TempDir::new().unwrap();
    let exe_path = temp_dir.path().join("test.exe");
    std::fs::write(&exe_path, b"fake exe").unwrap();

    let manager = Arc::new(
        crate::sqliterepository::sqliterepository::SqliteRepositoryManager::new(Arc::new(
            test_db.pool.clone(),
        )),
    );
    let resolver = Arc::new(DirsSavePathResolver::default());
    let exe_path_str = exe_path.to_string_lossy().to_string();
    let expected_working_dir = temp_dir.path().display().to_string();

    let mut shell = MockShellLink::new();
    shell
        .expect_create_bulk()
        .withf(move |reqs| {
            reqs.len() == 1
                && reqs[0].target_path == exe_path_str
                && reqs[0].working_dir.as_deref() == Some(expected_working_dir.as_str())
        })
        .returning(|_| Ok(()));

    let windows = Arc::new(TestWindows::new(shell));
    let service = WorkRegistrationServiceImpl::new(manager, resolver, windows);

    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(1)],
        insert: WorkInsert {
            title: "Test Work".to_string(),
            path: Some(RegisterWorkPath::Exe {
                exe_path: exe_path.to_string_lossy().to_string(),
            }),
            egs_info: None,
            icon: None,
            thumbnail: None,
            parent_pack_dmm_key: None,
        },
    }];

    let results = service.register(requests).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn register_親パック関連付けが正しく実行される() {
    let test_db = TestDatabase::new().await.unwrap();
    let service = create_service(&test_db);
    let repo = test_db.sqlite_repository();

    let parent_key = domain::work_parent_pack::ParentPackKey {
        store_id: "store".to_string(),
        category: "game".to_string(),
        subcategory: "pack".to_string(),
    };

    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(1)],
        insert: WorkInsert {
            title: "Child Work".to_string(),
            path: None,
            egs_info: None,
            icon: None,
            thumbnail: None,
            parent_pack_dmm_key: Some(parent_key.clone()),
        },
    }];

    let results = service.register(requests).await.unwrap();
    assert_eq!(results.len(), 1);

    // 親パック関連付けが作成されていることを確認
    let mut r = repo.work_parent_packs();
    let found_parent = r.find_parent_key(results[0].work_id.clone()).await.unwrap();
    assert!(found_parent.is_some());
    assert_eq!(found_parent.unwrap(), parent_key);
}

#[tokio::test]
async fn register_親パック関連付けの再登録でもエラーにならない() {
    let test_db = TestDatabase::new().await.unwrap();
    let service = create_service(&test_db);
    let repo = test_db.sqlite_repository();

    let parent_key = domain::work_parent_pack::ParentPackKey {
        store_id: "store".to_string(),
        category: "game".to_string(),
        subcategory: "pack".to_string(),
    };

    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(1)],
        insert: WorkInsert {
            title: "Child Work".to_string(),
            path: None,
            egs_info: None,
            icon: None,
            thumbnail: None,
            parent_pack_dmm_key: Some(parent_key.clone()),
        },
    }];

    let first_results = service.register(requests.clone()).await.unwrap();
    assert_eq!(first_results.len(), 1);

    let second_results = service.register(requests).await.unwrap();
    assert_eq!(second_results.len(), 1);
    assert_eq!(second_results[0].work_id.value, first_results[0].work_id.value);

    let mut r = repo.work_parent_packs();
    let found_parent = r
        .find_parent_key(second_results[0].work_id.clone())
        .await
        .unwrap();
    assert!(found_parent.is_some());
    assert_eq!(found_parent.unwrap(), parent_key);
}

#[tokio::test]
async fn register_画像戦略_never_は適用されない() {
    let test_db = TestDatabase::new().await.unwrap();
    let service = create_service(&test_db);

    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(1)],
        insert: WorkInsert {
            title: "Test Work".to_string(),
            path: None,
            egs_info: None,
            icon: Some(ImageApply {
                strategy: ImageStrategy::Never,
                source: ImageSource::FromUrl("https://example.com/icon.png".to_string()),
            }),
            thumbnail: None,
            parent_pack_dmm_key: None,
        },
    }];

    let results = service.register(requests).await.unwrap();
    assert_eq!(results.len(), 1);

    // 画像キューには投入されていないことを確認
    let repo = test_db.sqlite_repository();
    let queue_items = repo.image_queue().list(true, 10).await.unwrap();
    assert_eq!(queue_items.len(), 0);
}

#[tokio::test]
async fn register_サムネイル戦略_only_if_missing_egs_から取得される() {
    let test_db = TestDatabase::new().await.unwrap();
    let service = create_service(&test_db);
    let repo = test_db.sqlite_repository();

    // EGS キャッシュにサムネイルURLを設定
    let egs_id = 123;
    {
        let mut r = repo.all_game_cache();
        r.update(vec![domain::all_game_cache::NewAllGameCacheOne::new(
            egs_id,
            "Test Game".to_string(),
            "https://example.com/thumbnail.png".to_string(),
        )])
        .await
        .unwrap();
    }

    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(egs_id)],
        insert: WorkInsert {
            title: "Test Work".to_string(),
            path: None,
            egs_info: None,
            icon: None,
            thumbnail: Some(ImageApply {
                strategy: ImageStrategy::OnlyIfMissing,
                source: ImageSource::FromEgs,
            }),
            parent_pack_dmm_key: None,
        },
    }];

    let results = service.register(requests).await.unwrap();
    assert_eq!(results.len(), 1);

    // 画像キューにサムネイルが投入されていることを確認
    let queue_items = repo.image_queue().list(true, 10).await.unwrap();
    assert_eq!(queue_items.len(), 1);
    assert!(queue_items[0].dst_path.contains("thumbnail"));
    assert_eq!(
        queue_items[0].src,
        "https://example.com/thumbnail.png".to_string()
    );
}
