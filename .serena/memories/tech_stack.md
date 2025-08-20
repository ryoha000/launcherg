# 技術スタック

## フロントエンド
- **Svelte 5.35.6** - UIフレームワーク（最新バージョン）
- **TypeScript 5.8.3** - 型安全な開発（厳格モード有効）
- **UnoCSS 66.3.3** - スタイリング（Tailwind風のユーティリティクラス）
- **Vite 7.0.4** - ビルドツール
- **@mateothegreat/svelte5-router** - ルーティング

## バックエンド
- **Tauri 2.0.0-beta** - デスクトップアプリフレームワーク
- **Rust (Edition 2021)** - バックエンド言語
- **SQLx 0.6 + SQLite** - データベース（非同期対応）
- **Axum 0.7.5** - 内部HTTPサーバー
- **Tokio** - 非同期ランタイム

## 通信・データ形式
- **ProtoBuf** - ブラウザ拡張機能との通信
- **JSON** - Tauri内部（frontend-backend）通信
- **Serde** - シリアライズ/デシリアライズ

## 開発ツール
- **ESLint (@antfu/eslint-config)** - コードリンター
- **svelte-check** - Svelteの型チェック
- **Cargo** - Rustパッケージマネージャー