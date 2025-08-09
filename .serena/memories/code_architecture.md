# アーキテクチャとディレクトリ構造

## バックエンド（Rust）- クリーンアーキテクチャ
- `src-tauri/src/domain/` - ドメインモデルとリポジトリインターフェース
- `src-tauri/src/infrastructure/` - 外部システムとの連携（DB、ファイルシステム、Windows API）
- `src-tauri/src/interface/` - Tauriコマンドハンドラー
- `src-tauri/src/usecase/` - ビジネスロジック
- `src-tauri/src/migrations/` - データベースマイグレーション

## フロントエンド（Svelte）
- `src/components/` - Svelteコンポーネント
- `src/lib/` - 共通ユーティリティとAPIクライアント
- `src/views/` - ページビュー
- `src/router/` - ルーティング設定
- `src/store/` - Svelteストア（状態管理）
- `src/layouts/` - レイアウトコンポーネント
- `src/types/` - TypeScript型定義

## その他
- `browser-extension/` - ブラウザ拡張機能
- `proto/` - ProtoBuf定義ファイル
- `scripts/` - ビルドスクリプト