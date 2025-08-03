# コーディングスタイルと規約

## 共通
- **Gitコミット**: 日本語でコンベンショナルコミット形式
  - 種別: feat, fix, refactor, docs, style, test, chore
  - 例: `feat: ゲーム検索機能を追加`

## TypeScript/JavaScript
- **ESLint設定**: @antfu/eslint-config ベース
- **インデント**: スペース2つ
- **セミコロン**: なし（ESLint設定による）
- **クォート**: シングルクォート優先
- **prefer-const**: オフ（letの使用を許可）
- **ignoreパターン**: script/, scripts/, src-tauri/gen/, *.md

## Svelte
- **Svelte 5**: 最新の構文を使用（$derived, $stateなど）
- **lang='ts'**: TypeScriptを使用
- **スタイリング**: UnoCSS（Tailwind風クラス）

## Rust
- **Edition**: 2021
- **エラーハンドリング**: anyhow, thiserror
- **非同期**: async-trait, tokio
- **derive**: derive-new, serde

## 特記事項
- Windows固有機能は条件付きコンパイル（`#[cfg(windows)]`）
- ブラウザ拡張機能ではconsole.logとno-newルールを無効化