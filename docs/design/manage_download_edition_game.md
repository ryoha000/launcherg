# DL版ゲーム管理機能デザインドキュメント

## 概要

Launchergに、DMMやDLsiteなどのデジタルダウンロード版ゲームを管理する機能を追加する。ユーザーが購入済みのDL版ゲームを一覧表示し、未インストール・インストール済みの状態管理を行う。

## 目標

- DL版で購入済みのゲームをライブラリに表示
- 未インストールの場合はInstallボタンを表示
- Installボタンで販売サイトのブラウザページを開く
- インストール済みの場合はPlayボタンを表示
- 認証情報を保持せず、ブラウザ操作によるデータ同期

## アーキテクチャ設計

### 1. データモデル拡張

#### DownloadEditionGame (新規ドメインモデル)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEditionGame {
    pub id: Id<DownloadEditionGame>,
    pub platform: DownloadPlatform,      // DMM, DLsite, etc.
    pub platform_game_id: String,       // プラットフォーム固有のゲームID
    pub game_title: String,
    pub game_title_ruby: String,
    pub brand_name: String,
    pub brand_name_ruby: String,
    pub purchase_date: Option<DateTime<Local>>,
    pub local_installation_status: InstallationStatus,
    pub local_exe_path: Option<String>,  // インストール後のパス
    pub local_lnk_path: Option<String>,  // ショートカットパス
    pub platform_url: String,           // 販売ページURL
    pub thumbnail_url: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadPlatform {
    DMM,
    DLsite,
    // 将来的に他のプラットフォームを追加
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationStatus {
    NotInstalled,    // 未インストール
    Installed,       // インストール済み
    Unknown,         // 状態不明
}
```

#### CollectionElement拡張
既存の`CollectionElement`に以下のフィールドを追加：
```rust
pub download_edition_id: Option<Id<DownloadEditionGame>>,
pub source_type: GameSourceType,  // Local, DownloadEdition

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameSourceType {
    Local,           // ローカル検出ゲーム
    DownloadEdition, // DL版ゲーム
}
```

### 2. データ同期機能

#### DownloadEditionSyncService
```rust
pub trait DownloadEditionSyncService {
    async fn sync_platform_library(&self, platform: DownloadPlatform) -> Result<Vec<DownloadEditionGame>>;
    async fn update_installation_status(&self, game_id: Id<DownloadEditionGame>) -> Result<()>;
}
```

実装方針：
- **ブラウザ自動化は行わない** - セキュリティとプライバシーを重視
- **手動インポート機能** - ユーザーがJSONファイルやCSVでライブラリをインポート
- **ローカル検出機能** - インストール済みゲームの自動検出

#### 同期フロー
1. **手動インポート**
   - ユーザーが購入済みゲームリストをJSONまたはCSVで用意
   - インポート機能でLaunchergに取り込み

2. **ローカルインストール検出**
   - 既知のインストールパス（DMMゲームプレイヤー、DLsitePlayなど）をスキャン
   - レジストリ検索でインストール済みゲームを検出
   - ユーザーが手動でパスを指定

### 3. UI/UX設計

#### ゲーム一覧画面での表示
- 既存の`CollectionElement`リストにDL版ゲームも統合表示
- ゲームカードに`source_type`に応じたバッジを表示
- ローカルゲーム: "LOCAL"バッジ
- DL版ゲーム: プラットフォーム名バッジ（"DMM", "DLsite"）

#### ボタン状態管理
```typescript
interface GameActionButton {
  type: 'play' | 'install' | 'disabled'
  label: string
  action: () => void
}

function getGameActionButton(game: CollectionElement): GameActionButton {
  if (game.sourceType === 'Local'
    || (game.sourceType === 'DownloadEdition'
      && game.downloadEdition?.localInstallationStatus === 'Installed')) {
    return {
      type: 'play',
      label: 'Play',
      action: () => launchGame(game)
    }
  }

  if (game.sourceType === 'DownloadEdition'
    && game.downloadEdition?.localInstallationStatus === 'NotInstalled') {
    return {
      type: 'install',
      label: 'Install',
      action: () => openPlatformPage(game.downloadEdition.platformUrl)
    }
  }

  return {
    type: 'disabled',
    label: 'Unknown',
    action: () => {}
  }
}
```

#### 新規UI要素
1. **DL版ライブラリ管理画面**
   - サイドバーに「DL版ライブラリ」セクションを追加
   - プラットフォーム別フィルタリング
   - インストール状態別フィルタリング

2. **インポート機能UI**
   - サイドバーの「インポート」メニューにDL版インポートを追加
   - ドラッグ&ドロップでJSONファイルインポート
   - プラットフォーム選択とプレビュー機能

3. **インストール検出UI**
   - 設定画面に「DL版ゲーム検出」セクションを追加
   - 自動検出の実行ボタン
   - 手動パス指定機能

### 4. 実装手順

#### Phase 1: データモデルとリポジトリ
1. `DownloadEditionGame`ドメインモデルの追加
2. データベースマイグレーション作成
3. リポジトリインターフェースと実装

#### Phase 2: 同期サービス
1. `DownloadEditionSyncService`の実装
2. JSONインポート機能
3. ローカルインストール検出機能

#### Phase 3: UI統合
1. 既存ゲーム一覧へのDL版ゲーム統合
2. Playボタン/Installボタンの状態管理
3. プラットフォームページ開く機能

#### Phase 4: ライブラリ管理UI
1. DL版ライブラリ専用画面
2. インポート/エクスポート機能
3. インストール検出UI

### 5. セキュリティとプライバシー

#### プライバシー保護
- **認証情報の非保存**: ログインクッキーやトークンは保存しない
- **ローカル処理**: すべてのデータ処理をローカルで完結
- **最小権限**: 必要最小限のファイルアクセス権限のみ

#### データ保護
- **暗号化**: 購入履歴などの機密情報はローカル暗号化
- **匿名化**: 不要な個人情報は保存しない
- **ユーザー制御**: データの削除・エクスポート機能を提供

### 6. 技術実装詳細

#### ファイル配置
```
src-tauri/src/
├── domain/
│   ├── download_edition.rs          # 新規: DL版ゲームドメイン
│   └── repository/
│       └── download_edition.rs      # 新規: リポジトリインターフェース
├── infrastructure/
│   └── repositoryimpl/
│       ├── download_edition.rs      # 新規: リポジトリ実装
│       └── models/
│           └── download_edition.rs  # 新規: DBモデル
├── usecase/
│   ├── download_edition.rs          # 新規: ユースケース
│   └── models/
│       └── download_edition.rs      # 新規: ユースケースモデル
└── interface/
    ├── command.rs                    # 既存: コマンド追加
    └── models/
        └── download_edition.rs       # 新規: APIモデル

src/
├── components/
│   ├── Work/
│   │   ├── InstallButton.svelte     # 新規: インストールボタン
│   │   └── PlatformBadge.svelte     # 新規: プラットフォームバッジ
│   └── DownloadEdition/
│       ├── ImportWizard.svelte      # 新規: インポートウィザード
│       ├── LibraryView.svelte       # 新規: DL版ライブラリ画面
│       └── DetectionSettings.svelte # 新規: 検出設定
├── lib/
│   └── downloadEdition/
│       ├── import.ts                # 新規: インポート機能
│       ├── detection.ts             # 新規: ローカル検出
│       └── types.ts                 # 新規: 型定義
└── store/
    └── downloadEdition.ts           # 新規: DL版ゲーム状態管理
```

#### データベーススキーマ
```sql
-- V3__download_edition.sql
CREATE TABLE download_edition_games (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    platform_game_id TEXT NOT NULL,
    game_title TEXT NOT NULL,
    game_title_ruby TEXT NOT NULL,
    brand_name TEXT NOT NULL,
    brand_name_ruby TEXT NOT NULL,
    purchase_date TIMESTAMP NULL,
    local_installation_status TEXT NOT NULL DEFAULT 'Unknown',
    local_exe_path TEXT NULL,
    local_lnk_path TEXT NULL,
    platform_url TEXT NOT NULL,
    thumbnail_url TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(platform, platform_game_id)
);

-- collection_elementsテーブルの拡張
ALTER TABLE collection_elements
ADD COLUMN download_edition_id TEXT NULL,
ADD COLUMN source_type TEXT NOT NULL DEFAULT 'Local',
ADD FOREIGN KEY (download_edition_id) REFERENCES download_edition_games(id);
```

## 将来的な拡張

### 自動同期機能
- ブラウザ拡張機能による購入情報の自動収集
- WebDriver APIを使った自動ライブラリ同期
- プラットフォームAPIの公式サポート時の連携

### 高度なインストール管理
- ゲームの自動アップデート検出
- インストール容量の管理
- バックアップ・復元機能

### プラットフォーム拡張
- Steam、Epic Games Store等の既存プラットフォーム対応
- 海外プラットフォーム（itch.io、GOG等）対応
- 同人・インディゲームプラットフォーム対応

## まとめ

このデザインは、ユーザーのプライバシーとセキュリティを最優先に、DL版ゲームを既存のローカルゲーム管理と統合する形で設計している。手動インポートとローカル検出を中心とした実装により、認証情報を保持することなく安全にDL版ゲームライブラリを管理できる。

実装は段階的に行い、まずは基本的なデータモデルと手動インポート機能から開始し、徐々にUI機能とローカル検出機能を追加していく予定である。
