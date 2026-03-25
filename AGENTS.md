# AGENTS ガイド

常に日本語で返答してください。そのうえで `.cursor/rules/` 以下の詳細ルールを適切なタイミングで参照しながら作業します。

## 参照すべきルールファイルとタイミング
- **プロジェクト全体のワークフローや `task` コマンドの選択が必要なとき**: 作業前に `.cursor/rules/00-project-workflow.mdc` を確認する。
- **Cloudflare Workers / D1 / R2 / Wrangler まわり、または `server/` 配下の設計・実装・レビューをするとき**: 作業前にプロジェクトローカル skill [`$cloudflare-free-tier-guard`](F:\workspace\launcherg\.codex\skills\cloudflare-free-tier-guard\SKILL.md) を参照し、無料枠を前提にした設計制約と既存の `server/` 構成を確認する。
- **Tauri や Native Messaging Host の Rust コードを編集するとき**: `src-tauri/` 配下を触る前に `.cursor/rules/10-rust-workflow.mdc` を読む。
- **ブラウザ拡張（background / content-scripts / shared）の TypeScript を変更するとき**: `browser-extension/` を編集する前に `.cursor/rules/20-extension-workflow.mdc` を確認する。
- **Tauri フロントエンド（Svelte/TypeScript）に手を入れるとき**: `src/` を編集する前に `.cursor/rules/30-tauri-frontend-workflow.mdc` を参照する。
- **ルーターやタブの定義・スキーマを更新するとき**: 関連ファイルを変更する前に `.cursor/rules/31-router-tabs-architecture.mdc` を読む。
- **TanStack Query やデータ取得ロジック（クエリ/ミューテーション）を追加・変更するとき**: `src/lib/data/` や関連コンポーネントに着手する前に `.cursor/rules/32-data-fetch-patterns.mdc` を確認する。

## ワークフローの心得
- 作業開始時に影響範囲を特定し、該当するルールファイルを先に読み返す。
- Cloudflare 関連の変更では、無料枠の圧迫要因（Workers リクエスト、CPU、D1 操作、R2 ストレージ/転送）を先に整理し、不要な有料寄り構成を増やさない。
- 変更が複数領域にまたがる場合は、関連するすべてのルールを確認して整合性を担保する。
- 既存ルールで扱えないケースに遭遇したら、着手前に期待される方針を確認・合意する。
