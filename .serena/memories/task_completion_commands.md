# タスク完了時に実行するコマンド

## 必須: コミット前の品質チェック
```bash
npm run precommit
```
これは以下を順番に実行:
1. `npm run check` - Svelteの型チェック
2. `npm run format` - ESLintでコードフォーマット
3. `npm run lint` - ESLintでリント

## TypeScript/フロントエンド
```bash
# 型チェック
npm run check

# ビルドエラーチェック
npm run build

# リント
npm run lint

# フォーマット
npm run format
```

## Rust/バックエンド
```bash
# 型チェック
cd src-tauri && cargo check

# ビルドエラーチェック
cd src-tauri && cargo build

# フォーマットチェック
cd src-tauri && cargo fmt -- --check

# リント
cd src-tauri && cargo clippy

# テスト実行
cd src-tauri && cargo test
```
