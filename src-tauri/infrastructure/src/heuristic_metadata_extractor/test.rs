use std::sync::Arc;

use domain::all_game_cache::AllGameCacheOne;
use domain::game_matcher::MockGameMatcher;
use domain::scan::{CandidateKind, MetadataExtractor, WorkCandidate, WorkCandidateOrResolvedWork};

use super::HeuristicMetadataExtractor;

fn wc<P: Into<std::path::PathBuf>>(p: P) -> WorkCandidate {
    WorkCandidate::new(p.into(), CandidateKind::Exe)
}

#[test]
fn 正常系_候補ヒット_最初の候補を採用する() {
    let mut mock = MockGameMatcher::new();
    mock.expect_find_candidates()
        .withf(|qs| qs == &vec!["pieces".to_string(), "pieces".to_string()])
        .times(1)
        .returning_st(|_| {
            vec![(
                AllGameCacheOne::new(27123, "pieces/渡り鳥のソムニウム".to_string()),
                0.95,
            )]
        });

    let extractor = HeuristicMetadataExtractor::new(Arc::new(mock));
    let c = wc("W:\\others\\software\\Whirlpool\\pieces\\pieces.exe");

    let res = extractor.enrich(c.clone()).unwrap();
    match res {
        WorkCandidateOrResolvedWork::Resolved(r) => {
            assert_eq!(r.title, "pieces/渡り鳥のソムニウム");
            assert_eq!(r.egs_id, 27123);
            assert_eq!(r.candidate, c);
            assert!((r.distance - 0.95).abs() < 1e-6);
        }
        _ => panic!("expected Resolved"),
    }
}

#[test]
fn 正常系_候補なし_そのままcandidateを返す() {
    let mut mock = MockGameMatcher::new();
    mock.expect_find_candidates()
        .withf(|qs| qs == &vec!["pieces".to_string(), "pieces".to_string()])
        .times(1)
        .returning_st(|_| vec![]);

    let extractor = HeuristicMetadataExtractor::new(Arc::new(mock));
    let c = wc("W:\\others\\software\\Whirlpool\\pieces\\pieces.exe");

    let res = extractor.enrich(c.clone()).unwrap();
    match res {
        WorkCandidateOrResolvedWork::Candidate(cc) => {
            assert_eq!(cc, c);
        }
        _ => panic!("expected Candidate"),
    }
}

#[test]
fn 抽出エラー時_マッチャは呼ばれずcandidateを返す() {
    let mut mock = MockGameMatcher::new();
    mock.expect_find_candidates().never();

    let extractor = HeuristicMetadataExtractor::new(Arc::new(mock));
    let c = wc("C:\\test\\install.exe");

    let res = extractor.enrich(c.clone()).unwrap();
    match res {
        WorkCandidateOrResolvedWork::Candidate(cc) => assert_eq!(cc, c),
        _ => panic!("expected Candidate"),
    }
}

#[test]
fn スキップ対象ファイル名時_マッチャは呼ばれずcandidateを返す() {
    let mut mock = MockGameMatcher::new();
    mock.expect_find_candidates().never();

    let extractor = HeuristicMetadataExtractor::new(Arc::new(mock));
    let c = wc("C:\\Program Files\\Game\\start.exe");

    let res = extractor.enrich(c.clone()).unwrap();
    match res {
        WorkCandidateOrResolvedWork::Candidate(cc) => assert_eq!(cc, c),
        _ => panic!("expected Candidate"),
    }
}

#[test]
fn 複数候補時_先頭候補を採用する() {
    let mut mock = MockGameMatcher::new();
    mock.expect_find_candidates()
        .withf(|qs| qs == &vec!["pieces".to_string(), "pieces".to_string()])
        .times(1)
        .returning_st(|_| {
            vec![
                (AllGameCacheOne::new(1, "A".to_string()), 0.8),
                (AllGameCacheOne::new(2, "B".to_string()), 0.7),
            ]
        });

    let extractor = HeuristicMetadataExtractor::new(Arc::new(mock));
    let c = wc("W:\\others\\software\\Whirlpool\\pieces\\pieces.exe");

    let res = extractor.enrich(c.clone()).unwrap();
    match res {
        WorkCandidateOrResolvedWork::Resolved(r) => {
            assert_eq!(r.title, "A");
            assert_eq!(r.egs_id, 1);
            assert_eq!(r.candidate, c);
            assert!((r.distance - 0.8).abs() < 1e-6);
        }
        _ => panic!("expected Resolved"),
    }
}
