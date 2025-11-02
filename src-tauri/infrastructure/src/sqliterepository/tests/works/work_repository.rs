use super::super::TestDatabase;
use chrono::{DateTime, Local, NaiveDateTime};
use domain::erogamescape::NewErogamescapeInformation;
use domain::repository::{
    dmm_work_pack::DmmPackRepository,
    erogamescape::ErogamescapeRepository,
    work_download_path::WorkDownloadPathRepository,
    work_like::WorkLikeRepository,
    work_omit::WorkOmitRepository,
    works::{DlsiteWorkRepository, DmmWorkRepository, WorkRepository},
    RepositoriesExt,
};
use domain::works::{NewDlsiteWork, NewDmmWork, NewWork, NewWorkLike};

#[tokio::test]
async fn work_upsert_and_find_by_title_正常系() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "検索テスト".into(),
        })
        .await
        .unwrap()
    };
    assert!(!work_id.value.is_empty());

    // タイトルで検索
    let found = {
        let mut r = repo.work();
        r.find_by_title("検索テスト").await.unwrap()
    };
    assert!(found.is_some());
    let w = found.unwrap();
    assert_eq!(w.id.value, work_id.value);
    assert_eq!(w.title, "検索テスト");

    // 存在しないタイトルで検索
    let not_found = {
        let mut r = repo.work();
        r.find_by_title("存在しないタイトル").await.unwrap()
    };
    assert!(not_found.is_none());
}

#[tokio::test]
async fn work_find_details_by_work_id_存在しない_空() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // 存在しない work_id で検索
    let not_found = {
        let mut r = repo.work();
        r.find_details_by_work_id(domain::StrId::new("999999".to_string()))
            .await
            .unwrap()
    };
    assert!(not_found.is_none());
}

#[tokio::test]
async fn work_find_details_by_work_id_正常系_各_join反映() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "詳細テスト".into(),
        })
        .await
        .unwrap()
    };

    // DMM work を追加
    let _dmm_id = repo
        .dmm_work()
        .upsert(&NewDmmWork {
            store_id: "DMM-SID-001".into(),
            category: "software".into(),
            subcategory: "game".into(),
            work_id: work_id.clone(),
        })
        .await
        .unwrap();

    // Dlsite work を追加
    let _dlsite_id = repo
        .dlsite_work()
        .upsert(&NewDlsiteWork {
            store_id: "RJ123456".into(),
            category: "software".into(),
            work_id: work_id.clone(),
        })
        .await
        .unwrap();

    // erogamescape_information を追加
    let egs_id = 12345i32;
    {
        let mut r = repo.erogamescape();
        r.upsert_information(&NewErogamescapeInformation {
            erogamescape_id: egs_id,
            gamename_ruby: "ゲーム名ルビ".into(),
            sellday: "2024-01-01".into(),
            is_nukige: false,
            brandname: "ブランド名".into(),
            brandname_ruby: "ブランド名ルビ".into(),
        })
        .await
        .unwrap();
    }

    // work_erogamescape_map を追加
    {
        let mut r = repo.work();
        r.upsert_erogamescape_map(work_id.clone(), egs_id)
            .await
            .unwrap();
    }

    // work_thumbnails を追加
    {
        let mut r = repo.work();
        r.upsert_work_thumbnail_size(work_id.clone(), 1920, 1080)
            .await
            .unwrap();
    }

    // work_plays を追加
    let play_at = NaiveDateTime::parse_from_str("2024-01-20 15:30:00", "%Y-%m-%d %H:%M:%S")
        .unwrap();
    let play_at_dt: DateTime<Local> = DateTime::from_naive_utc_and_offset(
        play_at,
        Local::now().offset().clone(),
    );
    {
        let mut r = repo.work();
        r.update_last_play_at_by_work_id(work_id.clone(), play_at_dt)
            .await
            .unwrap();
    }

    // work_omits を追加
    {
        let mut r = repo.work_omit();
        r.add(work_id.clone()).await.unwrap();
    }

    // dmm_work_packs を追加
    {
        let mut r = repo.dmm_pack();
        r.add(work_id.clone()).await.unwrap();
    }

    // work_download_paths を追加
    {
        let mut r = repo.work_download_path();
        r.add(work_id.clone(), "C:/downloads/test")
            .await
            .unwrap();
    }

    // work_likes を追加
    let like_at = NaiveDateTime::parse_from_str("2024-01-25 12:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap();
    let like_at_dt: DateTime<Local> = DateTime::from_naive_utc_and_offset(
        like_at,
        Local::now().offset().clone(),
    );
    {
        let mut r = repo.work_like();
        r.upsert(&NewWorkLike {
            work_id: work_id.clone(),
            like_at: like_at_dt,
        })
        .await
        .unwrap();
    }

    // find_details_by_work_id で取得
    let details = {
        let mut r = repo.work();
        r.find_details_by_work_id(work_id.clone()).await.unwrap()
    };
    assert!(details.is_some());
    let d = details.unwrap();

    // 基本情報
    assert_eq!(d.work.id.value, work_id.value);
    assert_eq!(d.work.title, "詳細テスト");

    // DMM
    let dmm = d.dmm.as_ref().expect("DMM should exist");
    assert_eq!(dmm.store_id, "DMM-SID-001");
    assert_eq!(dmm.category, "software");
    assert_eq!(dmm.subcategory, "game");

    // Dlsite
    let dlsite = d.dlsite.as_ref().expect("Dlsite should exist");
    assert_eq!(dlsite.store_id, "RJ123456");
    assert_eq!(dlsite.category, "software");

    // Erogamescape
    assert_eq!(d.erogamescape_id, Some(egs_id));
    let egs_info = d
        .erogamescape_information
        .as_ref()
        .expect("ErogamescapeInformation should exist");
    assert_eq!(egs_info.gamename_ruby, "ゲーム名ルビ");
    assert_eq!(egs_info.brandname, "ブランド名");
    assert_eq!(egs_info.brandname_ruby, "ブランド名ルビ");
    assert_eq!(egs_info.sellday, "2024-01-01");
    assert_eq!(egs_info.is_nukige, false);

    // Thumbnail
    let thumb = d.thumbnail_size.as_ref().expect("ThumbnailSize should exist");
    assert_eq!(thumb.width, 1920);
    assert_eq!(thumb.height, 1080);

    // Play
    assert!(d.last_play_at.is_some());
    let play_dt = d.last_play_at.unwrap();
    assert_eq!(
        play_dt.naive_utc(),
        play_at
    );

    // Omit
    assert!(d.is_omitted);

    // DMM Pack
    assert!(d.is_dmm_pack);

    // Download Path
    let path = d
        .latest_download_path
        .as_ref()
        .expect("latest_download_path should exist");
    assert_eq!(path.download_path, "C:/downloads/test");

    // Like
    let like = d.like.as_ref().expect("Like should exist");
    assert_eq!(like.work_id.value, work_id.value);
}

#[tokio::test]
async fn work_find_work_ids_by_erogamescape_ids_空配列は空() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // 空配列で検索
    let result = {
        let mut r = repo.work();
        r.find_work_ids_by_erogamescape_ids(&[])
            .await
            .unwrap()
    };
    assert!(result.is_empty());
}

#[tokio::test]
async fn work_find_work_ids_by_erogamescape_ids_複数混在() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を2つ作成
    let work_id1 = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "EGS-1".into(),
        })
        .await
        .unwrap()
    };
    let work_id2 = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "EGS-2".into(),
        })
        .await
        .unwrap()
    };

    // erogamescape_information を追加
    let egs_id1 = 10001i32;
    let egs_id2 = 10002i32;
    {
        let mut r = repo.erogamescape();
        r.upsert_information(&NewErogamescapeInformation {
            erogamescape_id: egs_id1,
            gamename_ruby: "ゲーム1".into(),
            sellday: "2024-01-01".into(),
            is_nukige: false,
            brandname: "ブランド1".into(),
            brandname_ruby: "ブランド1ルビ".into(),
        })
        .await
        .unwrap();
        r.upsert_information(&NewErogamescapeInformation {
            erogamescape_id: egs_id2,
            gamename_ruby: "ゲーム2".into(),
            sellday: "2024-01-02".into(),
            is_nukige: false,
            brandname: "ブランド2".into(),
            brandname_ruby: "ブランド2ルビ".into(),
        })
        .await
        .unwrap();
    }

    // work_erogamescape_map を追加
    {
        let mut r = repo.work();
        r.upsert_erogamescape_map(work_id1.clone(), egs_id1)
            .await
            .unwrap();
        r.upsert_erogamescape_map(work_id2.clone(), egs_id2)
            .await
            .unwrap();
    }

    // 存在するIDと存在しないIDを混在させて検索
    let result = {
        let mut r = repo.work();
        r.find_work_ids_by_erogamescape_ids(&[egs_id1, 99999, egs_id2])
            .await
            .unwrap()
    };

    // 存在する分のみ返る
    assert_eq!(result.len(), 2);
    let mut pairs: Vec<(i32, String)> = result
        .into_iter()
        .map(|(egs, wid)| (egs, wid.value))
        .collect();
    pairs.sort_by_key(|(egs, _)| *egs);

    assert_eq!(pairs[0], (egs_id1, work_id1.value.clone()));
    assert_eq!(pairs[1], (egs_id2, work_id2.value.clone()));
}

#[tokio::test]
async fn work_upsert_erogamescape_map_挿入と更新() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "EGS-MAP".into(),
        })
        .await
        .unwrap()
    };

    // erogamescape_information を追加
    let egs_id1 = 20001i32;
    let egs_id2 = 20002i32;
    {
        let mut r = repo.erogamescape();
        r.upsert_information(&NewErogamescapeInformation {
            erogamescape_id: egs_id1,
            gamename_ruby: "ゲーム1".into(),
            sellday: "2024-01-01".into(),
            is_nukige: false,
            brandname: "ブランド1".into(),
            brandname_ruby: "ブランド1ルビ".into(),
        })
        .await
        .unwrap();
        r.upsert_information(&NewErogamescapeInformation {
            erogamescape_id: egs_id2,
            gamename_ruby: "ゲーム2".into(),
            sellday: "2024-01-02".into(),
            is_nukige: false,
            brandname: "ブランド2".into(),
            brandname_ruby: "ブランド2ルビ".into(),
        })
        .await
        .unwrap();
    }

    // 初回挿入
    {
        let mut r = repo.work();
        r.upsert_erogamescape_map(work_id.clone(), egs_id1)
            .await
            .unwrap();
    }

    // マップが作成されたことを確認
    let details1 = {
        let mut r = repo.work();
        r.find_details_by_work_id(work_id.clone()).await.unwrap()
    };
    assert!(details1.is_some());
    assert_eq!(details1.unwrap().erogamescape_id, Some(egs_id1));

    // 同じ work_id で別の egs_id に更新
    {
        let mut r = repo.work();
        r.upsert_erogamescape_map(work_id.clone(), egs_id2)
            .await
            .unwrap();
    }

    // 更新されたことを確認
    let details2 = {
        let mut r = repo.work();
        r.find_details_by_work_id(work_id.clone()).await.unwrap()
    };
    assert!(details2.is_some());
    assert_eq!(details2.unwrap().erogamescape_id, Some(egs_id2));
}

#[tokio::test]
async fn work_list_work_ids_missing_thumbnail_size_欠損とnull検出() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を3つ作成
    let work_id1 = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "サムネイルなし".into(),
        })
        .await
        .unwrap()
    };
    let work_id2 = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "width_null".into(),
        })
        .await
        .unwrap()
    };
    let work_id3 = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "height_null".into(),
        })
        .await
        .unwrap()
    };
    let work_id4 = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "サムネイルあり".into(),
        })
        .await
        .unwrap()
    };

    // work_id2, work_id3: サムネイルなし（欠損として検出される）
    // 注: repositoryメソッドではNULLを挿入できないため、サムネイルなしのケースのみテスト

    // work_id4: 両方あり（欠損なし）
    {
        let mut r = repo.work();
        r.upsert_work_thumbnail_size(work_id4.clone(), 1920, 1080)
            .await
            .unwrap();
    }

    // 欠損を検出
    let missing = {
        let mut r = repo.work();
        r.list_work_ids_missing_thumbnail_size().await.unwrap()
    };

    // work_id1, work_id2, work_id3（サムネイルなし）が含まれる
    assert_eq!(missing.len(), 3);
    let mut missing_ids: Vec<String> = missing.iter().map(|id| id.value.clone()).collect();
    missing_ids.sort();
    assert!(missing_ids.contains(&work_id1.value));
    assert!(missing_ids.contains(&work_id2.value));
    assert!(missing_ids.contains(&work_id3.value));
    assert!(!missing_ids.contains(&work_id4.value));
}

#[tokio::test]
async fn work_upsert_work_thumbnail_size_挿入と更新() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "サムネイルテスト".into(),
        })
        .await
        .unwrap()
    };

    // 初回挿入
    {
        let mut r = repo.work();
        r.upsert_work_thumbnail_size(work_id.clone(), 1920, 1080)
            .await
            .unwrap();
    }

    // 値が正しく挿入されたことを確認
    let details = {
        let mut r = repo.work();
        r.find_details_by_work_id(work_id.clone()).await.unwrap()
    };
    let details = details.expect("Details should exist");
    let thumb = details.thumbnail_size.as_ref().expect("ThumbnailSize should exist");
    assert_eq!(thumb.width, 1920);
    assert_eq!(thumb.height, 1080);

    // 欠損リストに含まれないことを確認
    let missing = {
        let mut r = repo.work();
        r.list_work_ids_missing_thumbnail_size().await.unwrap()
    };
    assert!(!missing.iter().any(|id| id.value == work_id.value));

    // 更新
    {
        let mut r = repo.work();
        r.upsert_work_thumbnail_size(work_id.clone(), 2560, 1440)
            .await
            .unwrap();
    }

    // 値が更新されたことを確認
    let details_after = {
        let mut r = repo.work();
        r.find_details_by_work_id(work_id.clone()).await.unwrap()
    };
    let details_after = details_after.expect("Details should exist");
    let thumb2 = details_after.thumbnail_size.as_ref().expect("ThumbnailSize should exist");
    assert_eq!(thumb2.width, 2560);
    assert_eq!(thumb2.height, 1440);

    // まだ欠損リストに含まれないことを確認
    let missing = {
        let mut r = repo.work();
        r.list_work_ids_missing_thumbnail_size().await.unwrap()
    };
    assert!(!missing.iter().any(|id| id.value == work_id.value));
}

#[tokio::test]
async fn work_repository_delete_should_remove_work_and_cascade() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // 1) work を作成し、DMM を紐付け
    let work_id = repo
        .work()
        .upsert(&NewWork {
            title: "DEL-TARGET".into(),
        })
        .await
        .unwrap();
    let _ = repo
        .dmm_work()
        .upsert(&NewDmmWork {
            store_id: "SID-DEL".into(),
            category: "software".into(),
            subcategory: "game".into(),
            work_id: work_id.clone(),
        })
        .await
        .unwrap();

    // 2) works から削除
    {
        let mut w = repo.work();
        w.delete(work_id.clone()).await.unwrap();
    }

    // 3) 再検索で見つからないこと
    let found = repo.work().find_by_title("DEL-TARGET").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn list_all_details_dmm_only() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();
    {
        let mut dmm_repo = repo.dmm_work();
        // work を作成
        let work_id = repo
            .work()
            .upsert(&NewWork {
                title: "Title A".into(),
            })
            .await
            .unwrap();
        let _ = dmm_repo
            .upsert(&NewDmmWork {
                store_id: "SID-1".into(),
                category: "software".into(),
                subcategory: "game".into(),
                work_id,
            })
            .await
            .unwrap();
    }
    let list = {
        let mut work_repo = repo.work();
        work_repo.list_all_details().await.unwrap()
    };
    assert_eq!(list.len(), 1);
    let item = &list[0];
    assert_eq!(item.work.title, "Title A");
    let dmm = item.dmm.as_ref().expect("DMM should exist");
    assert_eq!(dmm.work_id.value, item.work.id.value);
    assert_eq!(dmm.store_id, "SID-1");
    assert_eq!(dmm.category, "software");
    assert_eq!(dmm.subcategory, "game");
    assert!(item.dlsite.is_none());
    assert!(!item.is_omitted);
    assert!(!item.is_dmm_pack);
}

#[tokio::test]
async fn list_all_details_dlsite_only() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "DL-ONLY".into(),
        })
        .await
        .unwrap()
    };

    // Dlsite work を追加
    let _dlsite_id = repo
        .dlsite_work()
        .upsert(&NewDlsiteWork {
            store_id: "RJ999999".into(),
            category: "doujin".into(),
            work_id: work_id.clone(),
        })
        .await
        .unwrap();

    // list_all_details で取得
    let list = {
        let mut r = repo.work();
        r.list_all_details().await.unwrap()
    };
    assert_eq!(list.len(), 1);
    let item = &list[0];
    assert_eq!(item.work.title, "DL-ONLY");
    assert_eq!(item.work.id.value, work_id.value);

    // Dlsite が存在する
    let dlsite = item.dlsite.as_ref().expect("Dlsite should exist");
    assert_eq!(dlsite.store_id, "RJ999999");
    assert_eq!(dlsite.category, "doujin");
    assert_eq!(dlsite.work_id.value, work_id.value);

    // DMM は存在しない
    assert!(item.dmm.is_none());
    assert!(!item.is_omitted);
    assert!(!item.is_dmm_pack);
}

#[tokio::test]
async fn list_all_details_全要素_集約マージ() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&NewWork {
            title: "全要素テスト".into(),
        })
        .await
        .unwrap()
    };

    // DMM work を追加
    let _dmm_id = repo
        .dmm_work()
        .upsert(&NewDmmWork {
            store_id: "DMM-FULL".into(),
            category: "software".into(),
            subcategory: "game".into(),
            work_id: work_id.clone(),
        })
        .await
        .unwrap();

    // Dlsite work を追加
    let _dlsite_id = repo
        .dlsite_work()
        .upsert(&NewDlsiteWork {
            store_id: "RJ-FULL".into(),
            category: "software".into(),
            work_id: work_id.clone(),
        })
        .await
        .unwrap();

    // erogamescape_information を追加
    let egs_id = 99999i32;
    {
        let mut r = repo.erogamescape();
        r.upsert_information(&NewErogamescapeInformation {
            erogamescape_id: egs_id,
            gamename_ruby: "全要素ゲーム".into(),
            sellday: "2024-12-31".into(),
            is_nukige: true,
            brandname: "全要素ブランド".into(),
            brandname_ruby: "全要素ブランドルビ".into(),
        })
        .await
        .unwrap();
    }

    // work_erogamescape_map を追加
    {
        let mut r = repo.work();
        r.upsert_erogamescape_map(work_id.clone(), egs_id)
            .await
            .unwrap();
    }

    // work_thumbnails を追加
    {
        let mut r = repo.work();
        r.upsert_work_thumbnail_size(work_id.clone(), 3840, 2160)
            .await
            .unwrap();
    }

    // work_plays を追加
    let play_at = NaiveDateTime::parse_from_str("2024-06-15 20:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap();
    let play_at_dt: DateTime<Local> = DateTime::from_naive_utc_and_offset(
        play_at,
        Local::now().offset().clone(),
    );
    {
        let mut r = repo.work();
        r.update_last_play_at_by_work_id(work_id.clone(), play_at_dt)
            .await
            .unwrap();
    }

    // work_omits を追加
    {
        let mut r = repo.work_omit();
        r.add(work_id.clone()).await.unwrap();
    }

    // dmm_work_packs を追加
    {
        let mut r = repo.dmm_pack();
        r.add(work_id.clone()).await.unwrap();
    }

    // work_download_paths を追加（複数パスを追加して、最新のものが取得されることを確認）
    {
        let mut r = repo.work_download_path();
        r.add(work_id.clone(), "C:/downloads/old")
            .await
            .unwrap();
        r.add(work_id.clone(), "C:/downloads/latest")
            .await
            .unwrap();
    }

    // work_likes を追加
    let like_at = NaiveDateTime::parse_from_str("2024-07-01 12:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap();
    let like_at_dt: DateTime<Local> = DateTime::from_naive_utc_and_offset(
        like_at,
        Local::now().offset().clone(),
    );
    {
        let mut r = repo.work_like();
        r.upsert(&NewWorkLike {
            work_id: work_id.clone(),
            like_at: like_at_dt,
        })
        .await
        .unwrap();
    }

    // list_all_details で取得
    let list = {
        let mut r = repo.work();
        r.list_all_details().await.unwrap()
    };
    assert_eq!(list.len(), 1);
    let item = &list[0];

    // 基本情報
    assert_eq!(item.work.id.value, work_id.value);
    assert_eq!(item.work.title, "全要素テスト");

    // DMM
    let dmm = item.dmm.as_ref().expect("DMM should exist");
    assert_eq!(dmm.store_id, "DMM-FULL");
    assert_eq!(dmm.category, "software");
    assert_eq!(dmm.subcategory, "game");

    // Dlsite
    let dlsite = item.dlsite.as_ref().expect("Dlsite should exist");
    assert_eq!(dlsite.store_id, "RJ-FULL");
    assert_eq!(dlsite.category, "software");

    // Erogamescape
    assert_eq!(item.erogamescape_id, Some(egs_id));
    let egs_info = item
        .erogamescape_information
        .as_ref()
        .expect("ErogamescapeInformation should exist");
    assert_eq!(egs_info.gamename_ruby, "全要素ゲーム");
    assert_eq!(egs_info.brandname, "全要素ブランド");
    assert_eq!(egs_info.brandname_ruby, "全要素ブランドルビ");
    assert_eq!(egs_info.sellday, "2024-12-31");
    assert_eq!(egs_info.is_nukige, true);

    // Thumbnail
    let thumb = item
        .thumbnail_size
        .as_ref()
        .expect("ThumbnailSize should exist");
    assert_eq!(thumb.width, 3840);
    assert_eq!(thumb.height, 2160);

    // Play
    assert!(item.last_play_at.is_some());

    // Omit
    assert!(item.is_omitted);

    // DMM Pack
    assert!(item.is_dmm_pack);

    // Download Path（最新のものが取得される）
    let path = item
        .latest_download_path
        .as_ref()
        .expect("latest_download_path should exist");
    assert_eq!(path.download_path, "C:/downloads/latest");

    // Like
    let like = item.like.as_ref().expect("Like should exist");
    assert_eq!(like.work_id.value, work_id.value);
}

#[tokio::test]
async fn test_update_last_play_at_by_work_id_upsert() {
    use chrono::{DateTime, Local, NaiveDateTime};

    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // Work を作成
    let work_id = {
        let mut r = repo.work();
        r.upsert(&domain::works::NewWork::new("プレイ日時更新テスト".into()))
            .await
            .unwrap()
    };

    // 初回 INSERT: last_play_at を設定
    let first_play_at = NaiveDateTime::parse_from_str("2024-01-15 10:00:00", "%Y-%m-%d %H:%M:%S")
        .unwrap();
    let first_play_at_dt: DateTime<Local> = DateTime::from_naive_utc_and_offset(
        first_play_at,
        Local::now().offset().clone(),
    );

    {
        let mut r = repo.work();
        r.update_last_play_at_by_work_id(work_id.clone(), first_play_at_dt)
            .await
            .unwrap();
    }

    // 初回 INSERT が正しく保存されたことを確認
    let details = {
        let mut r = repo.work();
        r.find_details_by_work_id(work_id.clone()).await.unwrap().unwrap()
    };
    assert!(details.last_play_at.is_some());
    let saved_first = details.last_play_at.unwrap();
    assert_eq!(
        saved_first.naive_utc(),
        first_play_at,
        "初回の last_play_at が正しく保存されていること"
    );

    // 2回目 UPDATE: last_play_at を更新
    let second_play_at = NaiveDateTime::parse_from_str("2024-01-20 15:30:00", "%Y-%m-%d %H:%M:%S")
        .unwrap();
    let second_play_at_dt: DateTime<Local> = DateTime::from_naive_utc_and_offset(
        second_play_at,
        Local::now().offset().clone(),
    );

    {
        let mut r = repo.work();
        r.update_last_play_at_by_work_id(work_id.clone(), second_play_at_dt)
            .await
            .unwrap();
    }

    // UPDATE が正しく反映されたことを確認
    let details_after = {
        let mut r = repo.work();
        r.find_details_by_work_id(work_id.clone()).await.unwrap().unwrap()
    };
    assert!(details_after.last_play_at.is_some());
    let saved_second = details_after.last_play_at.unwrap();
    assert_eq!(
        saved_second.naive_utc(),
        second_play_at,
        "更新後の last_play_at が正しく保存されていること"
    );
    assert_ne!(
        saved_first.naive_utc(),
        saved_second.naive_utc(),
        "初回と2回目で last_play_at が異なること"
    );
}

