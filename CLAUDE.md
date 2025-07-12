# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

Launchergは、Tauriベースのデスクトップアプリケーションで、ゲームの管理と起動を行うランチャーです。

## 技術スタック

### フロントエンド
- **Svelte 3.54.0** - UIフレームワーク
- **TypeScript** - 型安全な開発
- **UnoCSS** - スタイリング
- **Vite** - ビルドツール

### バックエンド
- **Tauri 2.0.0-beta** - デスクトップアプリフレームワーク
- **Rust** - バックエンド言語
- **SQLx + SQLite** - データベース
- **Axum** - 内部HTTPサーバー

## CLIで実行するコマンド

**重要**: Claude CodeはCLIで完結するコマンドのみを実行します。アプリケーションの起動（`npm run dev`、`npm run tauri dev`など）は実行しません。

## Gitコミットルール

**重要**: 各作業の後には必ず日本語でコミットメッセージを作成してください。コミットメッセージは以下の形式に従ってください：

```
<種別>: <要約>

<詳細な説明（必要に応じて）>
```

種別の例：
- `feat`: 新機能
- `fix`: バグ修正
- `refactor`: リファクタリング
- `docs`: ドキュメント更新
- `style`: コードスタイルの変更
- `test`: テストの追加・修正
- `chore`: ビルドプロセスや補助ツールの変更

例：
```
feat: ゲーム検索機能を追加

- キーワード検索機能を実装
- ジャンル別フィルタリングを追加
- 検索結果のソート機能を実装
```

### コード品質チェック
```bash
# TypeScriptの型チェック
npm run check

# フロントエンドのビルド（エラーチェック用）
npm run build

# Rustコードのチェック
cd src-tauri && cargo check

# Rustコードのビルド（エラーチェック用）
cd src-tauri && cargo build

# Rustのフォーマットチェック
cd src-tauri && cargo fmt -- --check

# Rustのリント
cd src-tauri && cargo clippy
```

### データベース関連
```bash
# マイグレーションの実行（src-tauriディレクトリで実行）
cd src-tauri && cargo sqlx migrate run

# SQLxのオフラインデータ準備
cd src-tauri && cargo sqlx prepare
```

### その他のビルドコマンド
```bash
# アイコンの生成
npm run tauri icon public/icon.png

# プロダクションビルド（実行はしない）
npm run tauri build
```

## アーキテクチャ

### バックエンド構造（Rust）
クリーンアーキテクチャパターンを採用：
- `src-tauri/src/domain/` - ドメインモデルとリポジトリインターフェース
- `src-tauri/src/infrastructure/` - 外部システムとの連携（DB、ファイルシステム）
- `src-tauri/src/interface/` - Tauriコマンドハンドラー
- `src-tauri/src/usecase/` - ビジネスロジック

### フロントエンド構造
- `src/components/` - Svelteコンポーネント
- `src/lib/` - 共通ユーティリティとAPIクライアント
- `src/routes/` - ページコンポーネント
- `src/stores/` - Svelteストア（状態管理）

### 主要機能
- ゲームコレクション管理（GameExplorer, GameGrid）
- ファイルシステム探索とネットワーク機能
- スクリーンショット機能（Windows専用）
- プロセス管理
- QRコード生成（モバイル連携）
- WebRTC統合（SkyWay SDK）
- 自動アップデート機能

## 重要な注意事項

### Windows固有の機能
- スクリーンショット機能は`windows-capture`クレートを使用
- プロセス管理にWindows APIを使用

### セキュリティ
- `tauri.conf.json`の`dangerousDisableAssetCspModification`が有効
- アセットプロトコルが有効（ファイルアクセス用）

### データベース
- SQLiteを使用、マイグレーションは`src-tauri/migrations/`に配置
- SQLxのオフラインモード（`.sqlx/`ディレクトリ）を使用

### 開発時の注意
- TypeScriptは厳格モード有効
- フロントエンドのコード変更時は`npm run dev`で自動リロード
- Rustコードの変更時は`npm run tauri dev`の再起動が必要
