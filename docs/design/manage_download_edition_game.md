# DL版ゲーム管理機能デザインドキュメント

## 概要

Launchergにおいて、DMMやDLsiteで購入したダウンロード版ゲームを管理し、ユーザーが購入済みゲームのインストール・起動を効率的に行えるようにする機能を設計する。

## 目標

- 購入済みDL版ゲームのリスト表示
- 未インストールゲームに対するInstallボタンの提供
- インストール済みゲームに対するPlayボタンの提供
- ブラウザベースでのデータ同期（認証情報を保持しない）

## システム要件

### 機能要件

1. **DL版ゲーム情報の管理**
   - 購入済みゲームのメタデータ保存
   - インストール状態の追跡
   - 販売サイト情報の関連付け

2. **UI/UX**
   - 既存のPlayボタンと統一された見た目のInstallボタン
   - インストール状態に応じたボタンの自動切り替え
   - 購入済みゲームの視覚的識別

3. **ブラウザ連携**
   - DMMやDLsiteのゲームページへの直接リンク
   - ユーザーが手動でダウンロード・インストールを実行

### 非機能要件

- セキュリティ：認証情報を保持しない
- プライバシー：ユーザーの購入履歴は本人が管理
- 拡張性：新しい販売サイトの追加が容易

## アーキテクチャ設計

### データモデル

#### DLStoreGame（新規）
```rust
pub struct DLStoreGame {
    pub id: i64,
    pub title: String,
    pub store_id: String,     // DMMやDLsiteでのゲームID
    pub store_type: DLStoreType,
    pub purchase_url: String,  // ゲームページのURL
    pub install_path: Option<String>, // インストール済みの場合のパス
    pub is_installed: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub enum DLStoreType {
    DMM,
    DLSite,
}
```

#### CollectionElement拡張
既存のCollectionElementにDL版情報を関連付け：
```rust
pub struct CollectionElement {
    // 既存フィールド...
    pub dl_store_game_id: Option<i64>, // DLStoreGameとの関連
}
```

### データベース設計

#### 新規テーブル：dl_store_games
```sql
CREATE TABLE dl_store_games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    store_id TEXT NOT NULL,
    store_type TEXT NOT NULL, -- 'DMM' or 'DLSite'
    purchase_url TEXT NOT NULL,
    install_path TEXT,
    is_installed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(store_id, store_type)
);
```

#### collection_elements テーブル拡張
```sql
ALTER TABLE collection_elements
ADD COLUMN dl_store_game_id INTEGER REFERENCES dl_store_games(id);
```

### バックエンド実装

#### Repository層
- `DLStoreGameRepository`: DL版ゲーム情報のCRUD操作
- `CollectionRepository`拡張: DL版情報との関連付け

#### UseCase層
- `DLStoreGameUseCase`: DL版ゲームの管理ロジック
- `CollectionUseCase`拡張: 統合されたゲーム表示ロジック

#### Interface層
- Tauriコマンドでフロントエンドとの連携
- ブラウザ起動機能の提供

### フロントエンド実装

#### コンポーネント拡張

1. **PlayButton.svelte → GameActionButton.svelte**
   - PlayボタンとInstallボタンを統合
   - ゲームの状態に応じた表示切り替え

2. **Work.svelte拡張**
   - DL版情報の表示
   - インストール状態の視覚的表示

#### 新規コンポーネント

1. **DLStoreGameManager.svelte**
   - DL版ゲーム一覧の管理
   - ブラウザでのデータ同期UI

2. **InstallButton.svelte**
   - Installボタンの実装
   - ブラウザページ起動機能

### データ同期フロー

1. **初回セットアップ**
   - ユーザーがブラウザでDMM/DLsiteにログイン
   - 購入済みゲーム情報を手動でインポート（JSON/CSVファイル経由）

2. **定期同期**
   - ユーザーが手動でブラウザから購入履歴を確認
   - 新規購入ゲームの手動追加

3. **インストール検出**
   - ファイルシステムの変更監視
   - ユーザーによる手動パス指定

## 実装段階

### Phase 1: 基盤実装
- データモデル・スキーマ作成
- Repository・UseCase実装
- 基本的なCRUD操作

### Phase 2: UI実装
- GameActionButtonコンポーネント
- DL版ゲーム管理画面
- ブラウザ連携機能

### Phase 3: 統合・テスト
- 既存機能との統合
- エンドツーエンドテスト
- ユーザビリティ改善

## セキュリティ考慮事項

- 認証情報は一切保存しない
- ブラウザ起動時はユーザーの明示的な操作のみ
- ファイルパス情報は暗号化して保存（将来的検討）

## 今後の拡張可能性

- Steam、Epic Games Store等の対応
- 自動インストール検出の精度向上
- クラウド同期機能（オプション）
- 統計・レポート機能

## リスク

- ブラウザAPI変更によるデータ取得方法の影響
- 販売サイト側の仕様変更
- ユーザーの手動操作に依存する部分の使いやすさ

## 結論

本設計により、セキュアかつ拡張可能なDL版ゲーム管理機能を実現する。ユーザーのプライバシーを保護しながら、購入済みゲームの効率的な管理を可能にする。
