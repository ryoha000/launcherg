## プロジェクト概要

Launchergは、Tauriベースのデスクトップアプリケーションで、ゲームの管理と起動を行うランチャーです。

## 通信プロトコル

- 拡張機能 <-> native-messaging-host の通信する内容は ProtoBuf で定義されたものを使い、tauri 内の frontend(TypeScript) <-> backend(Rust) はJSONでやり取りする。

## CLIで実行するコマンド

CLIで完結するコマンドのみを実行すること
アプリケーションの起動（`npm run dev`、`npm run tauri dev`など）は実行しないこと

### データベース
- SQLiteを使用、マイグレーションは`src-tauri/migrations/`に配置
