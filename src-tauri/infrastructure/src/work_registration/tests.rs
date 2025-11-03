use std::sync::Arc;
use tempfile::TempDir;

use domain::repository::{
    all_game_cache::AllGameCacheRepository,
    save_image_queue::ImageSaveQueueRepository,
    work_parent_packs::WorkParentPacksRepository,
    works::WorkRepository,
    RepositoriesExt,
};
use domain::service::save_path_resolver::{DirsSavePathResolver, SavePathResolver};
use domain::service::work_registration::{
    ImageApply, ImageSource, ImageStrategy, RegisterWorkPath, UniqueWorkKey, WorkInsert,
    WorkRegistrationRequest, WorkRegistrationService,
};
use domain::works::NewWork;
use crate::sqliterepository::tests::TestDatabase;
use crate::sqliterepository::sqliterepository::{SqliteRepositoryManager, SqliteRepositories};
use crate::windowsimpl::windows::Windows;

use super::WorkRegistrationServiceImpl;

fn create_service(test_db: &TestDatabase) -> WorkRegistrationServiceImpl<
    SqliteRepositoryManager,
    SqliteRepositories,
    Windows,
> {
    let manager = Arc::new(
        crate::sqliterepository::sqliterepository::SqliteRepositoryManager::new(
            Arc::new(test_db.pool.clone()),
        ),
    );
    let resolver = Arc::new(DirsSavePathResolver::default());
    let windows = Arc::new(Windows::new());
    WorkRegistrationServiceImpl::new(manager, resolver, windows)
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
            parent_pack_work_id: None,
        },
    }];

    let results = service.register(requests).await.unwrap();
    assert_eq!(results.len(), 1);

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
            parent_pack_work_id: None,
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
        crate::sqliterepository::sqliterepository::SqliteRepositoryManager::new(
            Arc::new(test_db.pool.clone()),
        ),
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
            parent_pack_work_id: None,
        },
    }];

    let results = service.register(requests.clone()).await.unwrap();
    assert_eq!(results.len(), 1);

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
        crate::sqliterepository::sqliterepository::SqliteRepositoryManager::new(
            Arc::new(test_db.pool.clone()),
        ),
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
            parent_pack_work_id: None,
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
async fn register_親パック関連付けが正しく実行される() {
    let test_db = TestDatabase::new().await.unwrap();
    let service = create_service(&test_db);
    let repo = test_db.sqlite_repository();

    // 親 Work を作成
    let parent_work = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "Parent".to_string(),
        })
        .await
        .unwrap()
    };

    let requests = vec![WorkRegistrationRequest {
        keys: vec![UniqueWorkKey::ErogamescapeId(1)],
        insert: WorkInsert {
            title: "Child Work".to_string(),
            path: None,
            egs_info: None,
            icon: None,
            thumbnail: None,
            parent_pack_work_id: Some(parent_work.clone()),
        },
    }];

    let results = service.register(requests).await.unwrap();
    assert_eq!(results.len(), 1);

    // 親パック関連付けが作成されていることを確認
    let mut r = repo.work_parent_packs();
    let found_parent = r.find_parent_id(results[0].work_id.clone()).await.unwrap();
    assert!(found_parent.is_some());
    assert_eq!(found_parent.unwrap().value, parent_work.value);
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
            parent_pack_work_id: None,
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
            parent_pack_work_id: None,
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

