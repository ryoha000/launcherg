use std::path::Path;

/// 画像ソースの解決結果
#[derive(Debug)]
pub enum SourceDecision {
    /// 前処理に進むべきローカルソース
    Use(LocalSource),
    /// 既定アイコンを書いてスキップ（呼び出し側が既定アイコン出力を行う）。
    /// また、なぜフォールバックになったかの理由文字列を含む。
    FallbackDefaultAndSkip { reason: String },
}

/// ローカルソースと後始末戦略
#[derive(Debug)]
pub struct LocalSource {
    pub path: String,
    _cleanup: Cleanup,
}

impl LocalSource {
    pub fn new<P: Into<String>>(path: P, cleanup: Cleanup) -> Self {
        Self {
            path: path.into(),
            _cleanup: cleanup,
        }
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// 一時ファイルのクリーンアップ戦略
#[derive(Debug)]
pub enum Cleanup {
    None,
    /// Drop 時に指定パスを削除
    DeleteOnDrop {
        path: String,
    },
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        match self {
            Cleanup::None => {}
            Cleanup::DeleteOnDrop { path } => {
                let p = Path::new(path);
                if p.exists() {
                    let _ = std::fs::remove_file(p);
                }
            }
        }
    }
}
