use crate::integration_diagnostics::{
    record_diagnostics, IntegrationDiagnosticEvent, MAX_LOG_BYTES,
};

fn event() -> IntegrationDiagnosticEvent {
    IntegrationDiagnosticEvent {
        timestamp: "2026-10-24T05:00:00Z".into(),
        store: "dlsite".into(),
        stage: "sync_finished".into(),
        trigger: "auto".into(),
        page_kind: "library".into(),
        count: Some(3),
        detail: String::new(),
    }
}

#[test]
fn record_diagnostics_自動同期の経路を保存して容量上限でローテーションする() {
    let root = tempfile::tempdir().unwrap();
    record_diagnostics(root.path(), &[event()]).unwrap();
    let path = root.path().join("diagnostics/browser-sync.jsonl");
    let text = std::fs::read_to_string(&path).unwrap();
    let saved: IntegrationDiagnosticEvent = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(saved.trigger, "auto");
    assert_eq!(saved.count, Some(3));
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(MAX_LOG_BYTES)
        .unwrap();
    record_diagnostics(root.path(), &[event()]).unwrap();
    assert!(root
        .path()
        .join("diagnostics/browser-sync.previous.jsonl")
        .exists());
    assert!(std::fs::metadata(path).unwrap().len() < 1024);
}

#[test]
fn record_diagnostics_不正なイベントはファイルへ書き込まない() {
    let root = tempfile::tempdir().unwrap();
    let mut invalid = event();
    invalid.stage = "../private".into();
    assert!(record_diagnostics(root.path(), &[invalid]).is_err());
    assert!(!root.path().join("diagnostics").exists());
    assert!(record_diagnostics(root.path(), &[]).is_err());
}
