use domain::scan::{ResolvedWork, WorkCandidate, CandidateKind, DuplicateResolver};
use crate::heuristic_duplicate_resolver::HeuristicDuplicateResolver;

fn rw(path: &str, title: &str, egs_id: i32) -> ResolvedWork {
    rwd(path, title, egs_id, 0.5)
}

fn rwd(path: &str, title: &str, egs_id: i32, distance: f32) -> ResolvedWork {
    ResolvedWork {
        candidate: WorkCandidate { path: std::path::PathBuf::from(path), kind: CandidateKind::Exe },
        title: title.to_string(),
        egs_id,
        distance,
    }
}

#[test]
fn 単一要素はそのまま返す() {
    let sut = HeuristicDuplicateResolver;
    let input = vec![rw("C:/games/pieces/pieces.exe", "pieces", 100)];
    let out = sut.resolve(input.clone());
    assert_eq!(out, input);
}

#[test]
fn 異なるegs_idは各グループから1件ずつ返る() {
    let sut = HeuristicDuplicateResolver;
    let input = vec![
        rw("C:/games/a/title.exe", "title", 100),
        rw("C:/games/b/title.exe", "title", 200),
    ];
    let out = sut.resolve(input);
    // ユニーク egs_id 数だけ返る
    assert_eq!(out.len(), 2);
    // それぞれの egs_id を含む
    let mut ids: Vec<i32> = out.iter().map(|r| r.egs_id).collect();
    ids.sort();
    assert_eq!(ids, vec![100, 200]);
}

#[test]
fn ignore語が現ベストに含まれると挑戦者を採用() {
    // a = setup（IGNORE語）, b = 正常名 → b を採用
    let sut = HeuristicDuplicateResolver;
    let input = vec![
        rw("C:/tools/setup.exe", "pieces", 100),
        rw("C:/games/pieces/pieces.exe", "pieces", 100),
    ];
    let out = sut.resolve(input);
    assert_eq!(out.len(), 1);
    assert!(out[0].candidate.path.ends_with("pieces.exe"));
}

#[test]
fn ignore語が挑戦者に含まれると現ベスト維持() {
    // a = 正常名, b = setup（IGNORE語） → a を維持
    let sut = HeuristicDuplicateResolver;
    let input = vec![
        rw("C:/games/pieces/pieces.exe", "pieces", 100),
        rw("C:/tools/setup.exe", "pieces", 100),
    ];
    let out = sut.resolve(input);
    assert_eq!(out.len(), 1);
    assert!(out[0].candidate.path.ends_with("pieces.exe"));
}

#[test]
fn should_update語が挑戦者に含まれると採用() {
    // a = title, b = title64（SHOULD_UPDATE） → b を採用
    let sut = HeuristicDuplicateResolver;
    let input = vec![
        rw("C:/games/title/title.exe", "title", 100),
        rw("C:/games/title/title64.exe", "title", 100),
    ];
    let out = sut.resolve(input);
    assert_eq!(out.len(), 1);
    assert!(out[0].candidate.path.ends_with("title64.exe"));
}

#[test]
fn タイトル距離でより近いものを選ぶ() {
    // a = piecesx, b = pieces → b の方がタイトルに近い
    let sut = HeuristicDuplicateResolver;
    let input = vec![
        rwd("C:/games/pieces/piecesx.exe", "pieces", 100, 0.7),
        rwd("C:/games/pieces/pieces.exe", "pieces", 100, 0.9),
    ];
    let out = sut.resolve(input);
    assert_eq!(out.len(), 1);
    assert!(out[0].candidate.path.ends_with("pieces.exe"));
}

#[test]
fn 空のファイル名キーが混在してもパニックしない() {
    // a = ルートパスで file_stem 取得不能、b = 正常 → b を選ぶ
    let sut = HeuristicDuplicateResolver;
    let input = vec![
        rwd("C:/", "pieces", 100, 0.1),
        rwd("C:/games/pieces/pieces.exe", "pieces", 100, 0.9),
    ];
    let out = sut.resolve(input);
    assert_eq!(out.len(), 1);
    assert!(out[0].candidate.path.ends_with("pieces.exe"));
}


