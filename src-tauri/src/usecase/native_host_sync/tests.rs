use super::*;
use crate::domain::thumbnail::ThumbnailService;
use crate::domain::icon::IconService;
use async_trait::async_trait;
use mockall::predicate::*;
use std::sync::{Arc, Mutex};

use crate::{
    domain::{
        collection::CollectionElement,
        repository::collection::MockCollectionRepository,
        Id,
    },
    infrastructure::repositorymock::MockRepositoriesExtMock,
};

#[derive(Clone, Default)]
struct TestThumbnailService {
    // (collection_element_id, url)
    pub calls: Arc<Mutex<Vec<(i32, String)>>>,
}

#[async_trait]
impl ThumbnailService for TestThumbnailService {
    async fn save_thumbnail(&self, id: &Id<CollectionElement>, url: &str) -> anyhow::Result<()> {
        self.calls.lock().unwrap().push((id.value, url.to_string()));
        Ok(())
    }
}

#[derive(Clone, Default)]
struct TestIconService {
    pub calls: Arc<Mutex<Vec<(i32, String)>>>,
}

#[async_trait]
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
    thumbs: TestThumbnailService,
) -> (NativeHostSyncUseCase<MockRepositoriesExtMock, TestThumbnailService, TestIconService>, TestIconService) {
    let mut mock_repositories = MockRepositoriesExtMock::new();
    mock_repositories
        .expect_collection_repository()
        .return_const(mock_repo);

    let icons = TestIconService::default();
    (
        NativeHostSyncUseCase::new(
            Arc::new(mock_repositories),
            Arc::new(thumbs),
            Arc::new(icons.clone()),
        ),
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
async fn dmm_既存ならスキップ() {
    let mut repo = MockCollectionRepository::new();
    repo.expect_get_collection_id_by_dmm_mapping()
        .with(eq("sid"), eq("cat"), eq("sub"))
        .times(1)
        .returning(|_, _, _| Ok(Some(Id::<CollectionElement>::new(1))));

    let thumbs = TestThumbnailService::default();
    let (usecase, _icons) = new_usecase_with(repo, thumbs.clone());

    let params = vec![DmmSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        subcategory: "sub".into(),
        gamename: "Name".into(),
        egs: None,
        image_url: String::new(),
    }];

    let res = usecase.sync_dmm_games(params).await.unwrap();
    assert_eq!(res, 0);
    assert!(thumbs.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn dmm_egsあり_新規作成とサムネイル保存() {
    let mut repo = MockCollectionRepository::new();
    // no existing DMM mapping
    repo.expect_get_collection_id_by_dmm_mapping()
        .with(eq("sid"), eq("cat"), eq("sub"))
        .times(1)
        .returning(|_, _, _| Ok(None));
    // ensure_collection_for_egs path
    repo.expect_get_collection_id_by_erogamescape_id()
        .with(eq(42))
        .times(1)
        .returning(|_| Ok(None));
    repo.expect_allocate_new_collection_element_id()
        .with(eq("EGS Name"))
        .times(1)
        .returning(|_| Ok(Id::new(100)));
    repo.expect_upsert_erogamescape_map()
        .with(eq(Id::new(100)), eq(42))
        .times(1)
        .returning(|_, _| Ok(()));
    repo.expect_upsert_collection_element_info()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));
    // DMM mapping upsert
    repo.expect_upsert_dmm_mapping()
        .with(eq(Id::new(100)), eq("sid"), eq("cat"), eq("sub"))
        .times(1)
        .returning(|_, _, _, _| Ok(()));

    let thumbs = TestThumbnailService::default();
    let (usecase, icons) = new_usecase_with(repo, thumbs.clone());

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
    }];

    let res = usecase.sync_dmm_games(params).await.unwrap();
    assert_eq!(res, 1);
    let calls = thumbs.calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, 100);
    assert_eq!(calls[0].1, "https://pics.dmm.co.jp/digital/game/AAA/BBBpl.jpg");
    let icon_calls = icons.calls.lock().unwrap().clone();
    assert_eq!(icon_calls.len(), 1);
    assert_eq!(icon_calls[0].0, 100);
    assert_eq!(icon_calls[0].1, "https://pics.dmm.co.jp/digital/game/AAA/BBBps.jpg");
}

#[tokio::test]
async fn dmm_egsなし_新規作成のみ() {
    let mut repo = MockCollectionRepository::new();
    // no existing DMM mapping
    repo.expect_get_collection_id_by_dmm_mapping()
        .with(eq("sid"), eq("cat"), eq("sub"))
        .times(1)
        .returning(|_, _, _| Ok(None));
    // create_collection_without_egs path
    repo.expect_allocate_new_collection_element_id()
        .with(eq("Game Name"))
        .times(1)
        .returning(|_| Ok(Id::new(200)));
    // DMM mapping upsert
    repo.expect_upsert_dmm_mapping()
        .with(eq(Id::new(200)), eq("sid"), eq("cat"), eq("sub"))
        .times(1)
        .returning(|_, _, _, _| Ok(()));

    let thumbs = TestThumbnailService::default();
    let (usecase, icons) = new_usecase_with(repo, thumbs.clone());

    let params = vec![DmmSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        subcategory: "sub".into(),
        gamename: "Game Name".into(),
        egs: None,
        image_url: "https://pics.dmm.co.jp/digital/game/AAA/CCCps.jpg".into(),
    }];

    let res = usecase.sync_dmm_games(params).await.unwrap();
    assert_eq!(res, 1);
    let calls = thumbs.calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, 200);
    assert_eq!(calls[0].1, "https://pics.dmm.co.jp/digital/game/AAA/CCCpl.jpg");
    let icon_calls = icons.calls.lock().unwrap().clone();
    assert_eq!(icon_calls.len(), 1);
    assert_eq!(icon_calls[0].0, 200);
    assert_eq!(icon_calls[0].1, "https://pics.dmm.co.jp/digital/game/AAA/CCCps.jpg");
}

#[tokio::test]
async fn dlsite_既存ならスキップ() {
    let mut repo = MockCollectionRepository::new();
    repo.expect_get_collection_id_by_dlsite_mapping()
        .with(eq("sid"), eq("cat"))
        .times(1)
        .returning(|_, _| Ok(Some(Id::<CollectionElement>::new(10))));

    let thumbs = TestThumbnailService::default();
    let (usecase, _icons) = new_usecase_with(repo, thumbs.clone());

    let params = vec![DlsiteSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        gamename: "Name".into(),
        egs: None,
        image_url: String::new(),
    }];

    let res = usecase.sync_dlsite_games(params).await.unwrap();
    assert_eq!(res, 0);
    assert!(thumbs.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn dlsite_egsあり_新規作成とサムネイル保存() {
    let mut repo = MockCollectionRepository::new();
    // no existing DLsite mapping
    repo.expect_get_collection_id_by_dlsite_mapping()
        .with(eq("sid"), eq("cat"))
        .times(1)
        .returning(|_, _| Ok(None));
    // ensure_collection_for_egs path
    repo.expect_get_collection_id_by_erogamescape_id()
        .with(eq(77))
        .times(1)
        .returning(|_| Ok(None));
    repo.expect_allocate_new_collection_element_id()
        .with(eq("EGS Name DL"))
        .times(1)
        .returning(|_| Ok(Id::new(300)));
    repo.expect_upsert_erogamescape_map()
        .with(eq(Id::new(300)), eq(77))
        .times(1)
        .returning(|_, _| Ok(()));
    repo.expect_upsert_collection_element_info()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));
    // DLsite mapping upsert
    repo.expect_upsert_dlsite_mapping()
        .with(eq(Id::new(300)), eq("sid"), eq("cat"))
        .times(1)
        .returning(|_, _, _| Ok(()));

    let thumbs = TestThumbnailService::default();
    let (usecase, icons) = new_usecase_with(repo, thumbs.clone());

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
    let calls = thumbs.calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, 300);
    assert_eq!(
        calls[0].1,
        "https://img.dlsite.jp/modpub/images2/work/AAA/_img_main.jpg"
    );
    let icon_calls = icons.calls.lock().unwrap().clone();
    assert_eq!(icon_calls.len(), 1);
    assert_eq!(icon_calls[0].0, 300);
    assert_eq!(icon_calls[0].1, "https://img.dlsite.jp/resize/images2/work/AAA/_img_main_300x300.jpg");
}

#[tokio::test]
async fn dlsite_egsなし_新規作成のみ() {
    let mut repo = MockCollectionRepository::new();
    // no existing DLsite mapping
    repo.expect_get_collection_id_by_dlsite_mapping()
        .with(eq("sid"), eq("cat"))
        .times(1)
        .returning(|_, _| Ok(None));
    // create_collection_without_egs path
    repo.expect_allocate_new_collection_element_id()
        .with(eq("Game DL"))
        .times(1)
        .returning(|_| Ok(Id::new(400)));
    // DLsite mapping upsert
    repo.expect_upsert_dlsite_mapping()
        .with(eq(Id::new(400)), eq("sid"), eq("cat"))
        .times(1)
        .returning(|_, _, _| Ok(()));

    let thumbs = TestThumbnailService::default();
    let (usecase, icons) = new_usecase_with(repo, thumbs.clone());

    let params = vec![DlsiteSyncGameParam {
        store_id: "sid".into(),
        category: "cat".into(),
        gamename: "Game DL".into(),
        egs: None,
        image_url: "https://img.dlsite.jp/resize/images2/work/BBB/_img_main_300x300.jpg".into(),
    }];

    let res = usecase.sync_dlsite_games(params).await.unwrap();
    assert_eq!(res, 1);
    let calls = thumbs.calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, 400);
    assert_eq!(
        calls[0].1,
        "https://img.dlsite.jp/modpub/images2/work/BBB/_img_main.jpg"
    );
    let icon_calls = icons.calls.lock().unwrap().clone();
    assert_eq!(icon_calls.len(), 1);
    assert_eq!(icon_calls[0].0, 400);
    assert_eq!(icon_calls[0].1, "https://img.dlsite.jp/resize/images2/work/BBB/_img_main_300x300.jpg");
}


