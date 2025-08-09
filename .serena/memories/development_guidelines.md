# 開発ガイドライン

## 重要な注意事項

### Claude Code固有の制限
- CLIで完結するコマンドのみ実行
- アプリケーション起動コマンド（`npm run dev`、`npm run tauri dev`）は実行しない

### Windows固有の考慮事項
- スクリーンショット機能は`windows-capture`クレートを使用
- プロセス管理にWindows APIを使用
- 条件付きコンパイル（`#[cfg(windows)]`）を適切に使用

### セキュリティ
- `dangerousDisableAssetCspModification`が有効（tauri.conf.json）
- アセットプロトコルが有効（ファイルアクセス用）
- 秘密情報やキーをコミットしない

### 開発フロー
1. 機能実装・修正
2. `npm run precommit`で品質チェック
3. エラーがあれば修正
4. 日本語でコミットメッセージ作成

### データベース
- SQLiteを使用
- マイグレーションは`src-tauri/migrations/`に配置

### ブラウザ拡張機能
- ProtoBufで通信定義
- Native Messaging Host経由で通信
- Viteでビルド