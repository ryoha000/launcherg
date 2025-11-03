use crate::{erogamescape::NewErogamescapeInformation, works::Work, StrId};

/// Work の一意キーを表す enum
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UniqueWorkKey {
    ErogamescapeId(i32),
    Dmm {
        store_id: String,
        category: String,
        subcategory: String,
    },
    Dlsite {
        store_id: String,
        category: String,
    },
}

/// 手動登録時のパス種別
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegisterWorkPath {
    Exe { exe_path: String },
    Lnk { lnk_path: String },
}

/// 画像適用の戦略
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageStrategy {
    /// 常に適用
    Always,
    /// 新規 Work の場合のみ適用
    OnlyIfNew,
    /// 対象ファイルが存在しない場合のみ適用
    OnlyIfMissing,
    /// 適用しない
    Never,
}

/// 画像のソース
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageSource {
    /// URL から取得
    FromUrl(String),
    /// EGS キャッシュから取得
    FromEgs,
    /// パス（EXE/LNK）から抽出
    FromPath(RegisterWorkPath),
}

/// 画像適用の設定
#[derive(Clone, Debug)]
pub struct ImageApply {
    pub strategy: ImageStrategy,
    pub source: ImageSource,
}

/// Work 登録時に挿入する内容
#[derive(Clone, Debug)]
pub struct WorkInsert {
    pub title: String,
    pub path: Option<RegisterWorkPath>,
    pub egs_info: Option<NewErogamescapeInformation>,
    pub icon: Option<ImageApply>,
    pub thumbnail: Option<ImageApply>,
    pub parent_pack_work_id: Option<StrId<Work>>,
}

/// Work 登録リクエスト
#[derive(Clone, Debug)]
pub struct WorkRegistrationRequest {
    pub keys: Vec<UniqueWorkKey>,
    pub insert: WorkInsert,
}

/// Work 登録結果
#[derive(Clone, Debug)]
pub struct WorkRegistrationResult {
    pub resolved_keys: Vec<UniqueWorkKey>,
    pub work_id: StrId<Work>,
}

/// Work 登録サービス（単一API・バッチ対応）
#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkRegistrationService {
    /// Work を登録（バッチ対応、N+1回避）
    async fn register(
        &self,
        requests: Vec<WorkRegistrationRequest>,
    ) -> anyhow::Result<Vec<WorkRegistrationResult>>;
}

