# 推奨コマンド一覧

## 開発コマンド（実行しない）
- `npm run dev` - Viteの開発サーバー
- `npm run tauri dev` - Tauriアプリの開発モード

## ビルド・チェックコマンド
- `npm run precommit` - コミット前の品質チェック（必須）
- `npm run check` - TypeScriptの型チェック
- `npm run build` - フロントエンドのビルド
- `npm run lint` - ESLintでリント
- `npm run format` - ESLintでフォーマット
- `cd src-tauri && cargo check` - Rustコードのチェック
- `cd src-tauri && cargo build` - Rustコードのビルド
- `cd src-tauri && cargo clippy` - Rustのリント
- `cd src-tauri && cargo fmt` - Rustのフォーマット
- `cd src-tauri && cargo test` - Rustのテスト実行

## その他のビルドコマンド
- `npm run tauri build` - プロダクションビルド
- `npm run tauri icon public/icon.png` - アイコン生成
- `npm run build:native-host` - Native Messaging Hostのビルド
- `npm run cargo-test-local:proctail` - Windows固有テスト

## Windows固有コマンド
- `dir` - ディレクトリ内容表示（lsの代替）
- `type` - ファイル内容表示（catの代替）
- `findstr` - 文字列検索（grepの代替）
- `where` - コマンドの場所検索（whichの代替）