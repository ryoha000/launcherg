use super::*;
use domain::thumbnail::ThumbnailService;
use domain::icon::IconService;
use mockall::predicate::*;
use std::sync::{Arc, Mutex};

use domain::{
    collection::CollectionElement,
    repository::collection::MockCollectionRepository,
    repository::save_image_queue::MockImageSaveQueueRepository,
    repository::native_host_log::MockNativeHostLogRepository,
    Id,
};
use crate::repositorymock::TestRepositories;
use std::sync::Arc as StdArc;
use domain::repository::work_omit::MockWorkOmitRepository;

#[derive(Clone, Default)]
struct TestThumbnailService {
    // (collection_element_id, url)
    pub calls: Arc<Mutex<Vec<(i32, String)>>>,
}

impl ThumbnailService for TestThumbnailService {
    async fn save_thumbnail(&self, id: &Id<CollectionElement>, url: &str) -> anyhow::Result<()> {
        self.calls.lock().unwrap().push((id.value, url.to_string()));
        Ok(())
    }
    async fn get_thumbnail_size(&self, _id: &Id<CollectionElement>) -> anyhow::Result<Option<(u32, u32)>> {
        Ok(None)
    }
}

#[derive(Clone, Default)]
struct TestIconService {
    pub calls: Arc<Mutex<Vec<(i32, String)>>>,
}

impl IconService for TestIconService {
    async fn save_icon_from_path(&self, _id: &Id<CollectionElement>, _source_path: &str) -> anyhow::Result<()> { Ok(()) }
    async fn save_icon_from_url(&self, id: &Id<CollectionElement>, url: &str) -> anyhow::Result<()> {
        self.calls.lock().unwrap().push((id.value, url.to_string()));
        Ok(())
    }
    async fn save_default_icon(&self, _id: &Id<CollectionElement>) -> anyhow::Result<()> { Ok(()) }
}

fn new_usecase_with(
    mock_repo: MockCollectionRepository,
    expected_image_queue_calls: usize,
) -> (NativeHostSyncUseCase<crate::repositorymock::TestRepositoryManager, TestRepositories>, TestIconService) {
    let mut repos = TestRepositories::default().with_default_work_repos();
    repos.collection = StdArc::new(tauri::async_runtime::Mutex::new(mock_repo));

    // image queue enqueue は2回（アイコン+サムネイル）+ 作品別名サムネイル(1回) = 最大3回
    let mut imgq = MockImageSaveQueueRepository::new();
    imgq.expect_enqueue().times(expected_image_queue_calls).returning(|_, _, _, _| Box::pin(async move { Ok::<_, anyhow::Error>(Id::new(1)) }));
    repos.image_queue = StdArc::new(tauri::async_runtime::Mutex::new(imgq));

    // host log は呼ばれないためダミーを返す
    let hostlog = MockNativeHostLogRepository::new();
    repos.host_log = StdArc::new(tauri::async_runtime::Mutex::new(hostlog));

    // omit は exists=false を返すようにモック（全テストで共通仕様）
    let mut omit = MockWorkOmitRepository::new();
    omit.expect_exists().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(false) }));
    omit.expect_list().returning(|| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
    repos.work_omit = StdArc::new(tauri::async_runtime::Mutex::new(omit));

    let icons = TestIconService::default();
    (
        NativeHostSyncUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(repos)), Arc::new(DirsSavePathResolver::default())),
        icons,
    )
}

// ========== normalize_thumbnail_url ==========
#[test]
fn サムネイルurl正規化_dlsite() {
    let src = "https://img.dlsite.jp/resize/images2/work/doujin/RJ123/_img_main_300x300.jpg";
    let got = super::normalize_thumbnail_url(src);
    assert_eq!(
        got,
        "https://img.dlsite.jp/modpub/images2/work/doujin/RJ123/_img_main.jpg"
    );
}

#[test]
fn サムネイルurl正規化_dmm() {
    let src = "https://pics.dmm.co.jp/digital/game/AAA/BBBps.jpg";
    let got = super::normalize_thumbnail_url(src);
    assert_eq!(got, "https://pics.dmm.co.jp/digital/game/AAA/BBBpl.jpg");
}

#[tokio::test]
async fn 計画_decide_for_game_既存マッピングなら_skipexists() {
    // スナップショットに既存マッピングを入れる
    let key = DmmKey { store_id: "sid".into(), category: "cat".into(), subcategory: "sub".into() };
    let mut snapshot = DmmBatchSnapshot {
        work_id_by_key: Default::default(),
        mapped_keys: Default::default(),
        omitted_work_ids: Default::default(),
        egs_id_to_collection_id: Default::default(),
    };
    snapshot.mapped_keys.insert(key.clone(), Id::<CollectionElement>::new(1));

    // usecase を最小構成で生成
    let mock_repositories = TestRepositories::default().with_default_work_repos();
    let usecase = NativeHostSyncUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(mock_repositories)), Arc::new(DirsSavePathResolver::default()));

    let param = DmmSyncGameParam { store_id: "sid".into(), category: "cat".into(), subcategory: "sub".into(), gamename: "n".into(), egs: None, image_url: String::new(), parent_pack_work_id: None };
    let decided = usecase.decide_for_game(&snapshot, param).await.unwrap();
    match decided {
        PlanDecision::SkipExists => {},
        _ => panic!("expected SkipExists"),
    }
}

#[tokio::test]
async fn 計画_decide_for_game_omitがあれば_skipomitted() {
    // 既存マッピングはなし、work は存在する想定
    let key = DmmKey { store_id: "sid".into(), category: "cat".into(), subcategory: "sub".into() };
    let mut snapshot = DmmBatchSnapshot { work_id_by_key: Default::default(), mapped_keys: Default::default(), omitted_work_ids: Default::default(), egs_id_to_collection_id: Default::default() };
    snapshot.work_id_by_key.insert(key.clone(), Some(123));
    snapshot.omitted_work_ids.insert(123);

    // omit リポジトリを exists=true に設定
    let mut mock_repositories = TestRepositories::default().with_default_work_repos();
    let mut omit = MockWorkOmitRepository::new();
    omit.expect_exists().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(true) }));
    mock_repositories.work_omit = StdArc::new(tauri::async_runtime::Mutex::new(omit));
    let usecase = NativeHostSyncUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(mock_repositories)), Arc::new(DirsSavePathResolver::default()));

    let param = DmmSyncGameParam { store_id: "sid".into(), category: "cat".into(), subcategory: "sub".into(), gamename: "n".into(), egs: None, image_url: String::new(), parent_pack_work_id: None };
    let decided = usecase.decide_for_game(&snapshot, param).await.unwrap();
    match decided {
        PlanDecision::SkipOmitted => {},
        _ => panic!("expected SkipOmitted"),
    }
}

#[tokio::test]
async fn 実行_execute_apply_egsあり_採番とマッピングと画像() {
    use mockall::predicate::*;
    // collection の期待
    let mut repo = MockCollectionRepository::new();
    // ensure_collection_for_egs の流れ
    repo.expect_get_collection_id_by_erogamescape_id()
        .with(eq(42))
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(None) }));
    repo.expect_allocate_new_collection_element_id()
        .with(eq("EGS Name"))
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(Id::new(5555)) }));
    repo.expect_upsert_erogamescape_map()
        .with(eq(Id::new(5555)), eq(42))
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));
    repo.expect_upsert_collection_element_info()
        .with(always())
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));
    // work mapping
    repo.expect_upsert_work_mapping()
        .with(eq(Id::new(5555)), eq(1000))
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));

    // リポジトリ束
    let mut mock_repositories = TestRepositories::default().with_default_work_repos();
    mock_repositories.collection = StdArc::new(tauri::async_runtime::Mutex::new(repo));
    // 画像キューは3回
    let mut imgq = MockImageSaveQueueRepository::new();
    imgq.expect_enqueue().times(3).returning(|_, _, _, _| Box::pin(async move { Ok::<_, anyhow::Error>(Id::new(1)) }));
    mock_repositories.image_queue = StdArc::new(tauri::async_runtime::Mutex::new(imgq));

    let usecase = NativeHostSyncUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(mock_repositories)), Arc::new(DirsSavePathResolver::default()));

    // 実行
    let apply = SyncApply {
        key: DmmKey { store_id: "sid".into(), category: "cat".into(), subcategory: "sub".into() },
        work_id_opt: Some(1000),
        gamename: "ignore".into(),
        image_url: "https://pics.dmm.co.jp/digital/game/AAA/BBBps.jpg".into(),
        parent_pack_work_id: Some(2000),
        egs: Some(EgsInfo { erogamescape_id: 42, gamename: "EGS Name".into(), gamename_ruby: "r".into(), brandname: "b".into(), brandname_ruby: "br".into(), sellday: "2024".into(), is_nukige: false })
    };
    let mut caches = Caches::default();
    usecase.execute_apply(apply, &mut caches).await.unwrap();
}

#[tokio::test]
async fn 画像_enqueue_images_for_dmm_3回投入される() {
    // 3回 enqueue 期待
    let mut mock_repositories = TestRepositories::default().with_default_work_repos();
    let mut imgq = MockImageSaveQueueRepository::new();
    imgq.expect_enqueue().times(3).returning(|_, _, _, _| Box::pin(async move { Ok::<_, anyhow::Error>(Id::new(1)) }));
    mock_repositories.image_queue = StdArc::new(tauri::async_runtime::Mutex::new(imgq));
    // collection は未使用だが要求されるためダミーを返す
    let repo = MockCollectionRepository::new();
    mock_repositories.collection = StdArc::new(tauri::async_runtime::Mutex::new(repo));

    let usecase = NativeHostSyncUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(mock_repositories)), Arc::new(DirsSavePathResolver::default()));
    let id = Id::<CollectionElement>::new(999);
    usecase.enqueue_images_for_dmm(&id, "cat", "sub", "sid", "https://pics.dmm.co.jp/digital/game/AAA/BBBps.jpg").await.unwrap();
}

#[tokio::test]
async fn スナップショット_build_dmm_batch_snapshot_既存と未存在が取り込まれる() {
    use mockall::predicate::*;
    let mut repo = MockCollectionRepository::new();
    repo.expect_get_collection_ids_by_work_ids()
        .with(always())
        .times(1)
        .returning(|work_ids: &[i32]| {
            let out: Vec<(i32, Id<CollectionElement>)> = work_ids.iter().take(1).map(|wid| (*wid, Id::new(1))).collect();
            Box::pin(async move { Ok::<_, anyhow::Error>(out) })
        });

    let mut mock_repositories = TestRepositories::default().with_default_work_repos();
    mock_repositories.collection = StdArc::new(tauri::async_runtime::Mutex::new(repo));
    // omit リストは空
    let mut omit = MockWorkOmitRepository::new();
    omit.expect_list().returning(|| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
    mock_repositories.work_omit = StdArc::new(tauri::async_runtime::Mutex::new(omit));
    // image queue ダミー
    let mut imgq = MockImageSaveQueueRepository::new();
    imgq.expect_enqueue().times(0).returning(|_, _, _, _| Box::pin(async move { Ok::<_, anyhow::Error>(Id::new(1)) }));
    mock_repositories.image_queue = StdArc::new(tauri::async_runtime::Mutex::new(imgq));

    let usecase = NativeHostSyncUseCase::new(Arc::new(crate::repositorymock::TestRepositoryManager::new(mock_repositories)), Arc::new(DirsSavePathResolver::default()));
    let params = vec![
        DmmSyncGameParam { store_id: "sid1".into(), category: "cat".into(), subcategory: "sub".into(), gamename: "n1".into(), egs: None, image_url: String::new(), parent_pack_work_id: None },
        DmmSyncGameParam { store_id: "sid2".into(), category: "cat".into(), subcategory: "sub".into(), gamename: "n2".into(), egs: None, image_url: String::new(), parent_pack_work_id: None },
    ];
    let snapshot = usecase.build_dmm_batch_snapshot(&params).await.unwrap();
    // sid1 は mapped、両方に work_id が存在（with_default_work_repos は全てに Some を返す）
    assert!(snapshot.mapped_keys.contains_key(&DmmKey { store_id: "sid1".into(), category: "cat".into(), subcategory: "sub".into() }));
    assert!(snapshot.work_id_by_key.get(&DmmKey { store_id: "sid1".into(), category: "cat".into(), subcategory: "sub".into() }).unwrap().is_some());
    assert!(snapshot.work_id_by_key.get(&DmmKey { store_id: "sid2".into(), category: "cat".into(), subcategory: "sub".into() }).unwrap().is_some());
}
async fn dmm_既存ならスキップ() {
    let mut repo = MockCollectionRepository::new();
    // work_id ベースで既存マッピングありを返す
    repo.expect_get_collection_ids_by_work_ids()
        .with(always())
        .times(1)
        .returning(|work_ids: &[i32]| {
            let out: Vec<(i32, Id<CollectionElement>)> = work_ids.iter().take(1).map(|wid| (*wid, Id::new(1))).collect();
            Box::pin(async move { Ok::<_, anyhow::Error>(out) })
        });

    let thumbs = TestThumbnailService::default();
    let (usecase, _icons) = new_usecase_with(repo, 0);

    let params = vec![DmmSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        subcategory: "sub".into(),
        gamename: "Name".into(),
        egs: None,
        image_url: String::new(),
        parent_pack_work_id: None,
    }];

    let res = usecase.sync_dmm_games(params).await.unwrap();
    assert_eq!(res, 0);
    assert!(thumbs.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn dmm_egsあり_新規作成とサムネイル保存() {
    let mut repo = MockCollectionRepository::new();
    // work_id ベースで既存マッピングなし
    repo.expect_get_collection_ids_by_work_ids()
        .with(always())
        .times(1)
        .returning(|_work_ids| Box::pin(async move { Ok::<_, anyhow::Error>(Vec::new()) }));
    // EGS 一括は空を返す（プリフェッチ用）
    repo.expect_get_collection_ids_by_erogamescape_ids()
        .with(always())
        .times(1)
        .returning(|_ids: &[i32]| Box::pin(async move { Ok::<_, anyhow::Error>(Vec::new()) }));
    // ensure_collection_for_egs path
    repo.expect_get_collection_id_by_erogamescape_id()
        .with(eq(42))
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(None) }));
    repo.expect_allocate_new_collection_element_id()
        .with(eq("EGS Name"))
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(Id::new(100)) }));
    repo.expect_upsert_erogamescape_map()
        .with(eq(Id::new(100)), eq(42))
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));
    repo.expect_upsert_collection_element_info()
        .with(always())
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));
    // DMM mapping upsert -> work_mapping に変更（work_id は dmm_work_repository 経由で取得されるためここでは期待しない）
    repo.expect_upsert_work_mapping()
        .with(eq(Id::new(100)), always())
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));

    let (usecase, icons) = new_usecase_with(repo, 3);

    let params = vec![DmmSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        subcategory: "sub".into(),
        gamename: "ignored".into(),
        egs: Some(EgsInfo {
            erogamescape_id: 42,
            gamename: "EGS Name".into(),
            gamename_ruby: "ruby".into(),
            brandname: "brand".into(),
            brandname_ruby: "brand_ruby".into(),
            sellday: "2024-01-01".into(),
            is_nukige: false,
        }),
        image_url: "https://pics.dmm.co.jp/digital/game/AAA/BBBps.jpg".into(),
        parent_pack_work_id: None,
    }];

    let res = usecase.sync_dmm_games(params).await.unwrap();
    assert_eq!(res, 1);
    let icon_calls = icons.calls.lock().unwrap().clone();
    assert_eq!(icon_calls.len(), 0);
}

#[tokio::test]
async fn dmm_egsなし_新規作成のみ() {
    let mut repo = MockCollectionRepository::new();
    // work_id ベースで既存マッピングなし
    repo.expect_get_collection_ids_by_work_ids()
        .with(always())
        .times(1)
        .returning(|_work_ids| Box::pin(async move { Ok::<_, anyhow::Error>(Vec::new()) }));
    // create_collection_without_egs path
    repo.expect_allocate_new_collection_element_id()
        .with(eq("Game Name"))
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(Id::new(200)) }));
    // DMM mapping upsert -> work_mapping
    repo.expect_upsert_work_mapping()
        .with(eq(Id::new(200)), always())
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));

    let (usecase, icons) = new_usecase_with(repo, 3);

    let params = vec![DmmSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        subcategory: "sub".into(),
        gamename: "Game Name".into(),
        egs: None,
        image_url: "https://pics.dmm.co.jp/digital/game/AAA/CCCps.jpg".into(),
        parent_pack_work_id: None,
    }];

    let res = usecase.sync_dmm_games(params).await.unwrap();
    assert_eq!(res, 1);
    let icon_calls = icons.calls.lock().unwrap().clone();
    assert_eq!(icon_calls.len(), 0);
}

#[tokio::test]
async fn dlsite_既存ならスキップ() {
    let mut repo = MockCollectionRepository::new();
    repo.expect_get_collection_id_by_dlsite_mapping()
        .with(eq("sid"), eq("cat"))
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(Some(Id::<CollectionElement>::new(10))) }));

    let (usecase, _icons) = new_usecase_with(repo, 0);

    let params = vec![DlsiteSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        gamename: "Name".into(),
        egs: None,
        image_url: String::new(),
    }];

    let res = usecase.sync_dlsite_games(params).await.unwrap();
    assert_eq!(res, 0);
}

#[tokio::test]
async fn dlsite_egsあり_新規作成とサムネイル保存() {
    let mut repo = MockCollectionRepository::new();
    // no existing DLsite mapping
    repo.expect_get_collection_id_by_dlsite_mapping()
        .with(eq("sid"), eq("cat"))
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(None) }));
    // ensure_collection_for_egs path
    repo.expect_get_collection_id_by_erogamescape_id()
        .with(eq(77))
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(None) }));
    repo.expect_allocate_new_collection_element_id()
        .with(eq("EGS Name DL"))
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(Id::new(300)) }));
    repo.expect_upsert_erogamescape_map()
        .with(eq(Id::new(300)), eq(77))
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));
    repo.expect_upsert_collection_element_info()
        .with(always())
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));
    // DLsite mapping upsert -> work_mapping
    repo.expect_upsert_work_mapping()
        .with(eq(Id::new(300)), always())
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));

    let (usecase, icons) = new_usecase_with(repo, 3);

    let params = vec![DlsiteSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        gamename: "ignored".into(),
        egs: Some(EgsInfo {
            erogamescape_id: 77,
            gamename: "EGS Name DL".into(),
            gamename_ruby: "ruby".into(),
            brandname: "brand".into(),
            brandname_ruby: "brand_ruby".into(),
            sellday: "2024-02-02".into(),
            is_nukige: false,
        }),
        image_url: "https://img.dlsite.jp/resize/images2/work/AAA/_img_main_300x300.jpg".into(),
    }];

    let res = usecase.sync_dlsite_games(params).await.unwrap();
    assert_eq!(res, 1);
    let icon_calls = icons.calls.lock().unwrap().clone();
    assert_eq!(icon_calls.len(), 0);
}

#[tokio::test]
async fn dlsite_egsなし_新規作成のみ() {
    let mut repo = MockCollectionRepository::new();
    // no existing DLsite mapping
    repo.expect_get_collection_id_by_dlsite_mapping()
        .with(eq("sid"), eq("cat"))
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(None) }));
    // create_collection_without_egs path
    repo.expect_allocate_new_collection_element_id()
        .with(eq("Game DL"))
        .times(1)
        .returning(|_| Box::pin(async move { Ok::<_, anyhow::Error>(Id::new(400)) }));
    // DLsite mapping upsert -> work_mapping
    repo.expect_upsert_work_mapping()
        .with(eq(Id::new(400)), always())
        .times(1)
        .returning(|_, _| Box::pin(async move { Ok::<_, anyhow::Error>(()) }));

    let (usecase, icons) = new_usecase_with(repo, 3);

    let params = vec![DlsiteSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        gamename: "Game DL".into(),
        egs: None,
        image_url: "https://img.dlsite.jp/resize/images2/work/BBB/_img_main_300x300.jpg".into(),
    }];

    let res = usecase.sync_dlsite_games(params).await.unwrap();
    assert_eq!(res, 1);
    let icon_calls = icons.calls.lock().unwrap().clone();
    assert_eq!(icon_calls.len(), 0);
}


