# Native Messaging Host

このドキュメントは、Launchergブラウザ拡張機能とデスクトップアプリケーション間の通信を担うNative Messaging Hostについて説明します。

## 概要

Native Messaging Hostは、ブラウザ拡張機能とデスクトップアプリケーション間でセキュアな通信を行うための実行可能プログラムです。Chrome/Edge標準のNative Messaging APIを使用して、JSONメッセージをやり取りします。

## ビルド方法

```bash
# npmスクリプトを使用（推奨）
npm run build:native-host       # リリースビルド
npm run build:native-host:debug # デバッグビルド

# または直接cargoを使用
cd src-tauri
cargo build --release --bin native-messaging-host
```

実行ファイルは以下の場所に生成されます：
- デバッグ: `target/debug/native-messaging-host.exe`
- リリース: `target/release/native-messaging-host.exe`
- npmスクリプト使用時: `src-tauri/native-messaging-host.exe`にコピーされます

## インストール方法

### Windows

#### 方法1: アプリケーションからのインストール（推奨）

1. Native Messaging Hostをビルド
   ```bash
   npm run build:native-host
   ```

2. Launchergアプリケーションを起動

3. 設定 → ブラウザ拡張機能管理 を開く

4. Extension IDを設定（オプション）

5. "Native Messaging Host セットアップ"の"セットアップを開始"ボタンをクリック

#### 方法2: 手動インストール

アプリケーション内部でPowerShellスクリプトが自動生成・実行されるため、手動でのインストールは通常不要です。

### アンインストール

```powershell
.\uninstall-native-messaging-host.ps1
```

## テスト方法

### 単体テスト

```powershell
# PowerShellでテストスクリプトを実行
.\test-native-messaging-host.ps1

# カスタムパスの実行ファイルをテスト
.\test-native-messaging-host.ps1 -ExePath ".\target\release\native-messaging-host.exe"
```

### 手動テスト

1. Native Messaging Hostを直接起動
2. 標準入力にNative Messaging形式でメッセージを送信
3. 標準出力からレスポンスを読み取り

## プロトコル仕様

### メッセージ形式

#### リクエスト
```json
{
  "type": "sync_games | get_status | set_config | health_check",
  "payload": {},
  "timestamp": "2025-01-30T12:34:56Z",
  "request_id": "unique-request-id"
}
```

#### レスポンス
```json
{
  "success": true,
  "data": {},
  "error": null,
  "request_id": "unique-request-id"
}
```

### メッセージタイプ

#### sync_games
ゲーム情報の同期リクエスト

リクエスト:
```json
{
  "type": "sync_games",
  "payload": {
    "store": "DMM",
    "games": [
      {
        "store_id": "game_001",
        "title": "Game Title",
        "purchase_url": "https://...",
        "purchase_date": "2025-01-30",
        "thumbnail_url": "https://...",
        "additional_data": {
          "erogamescape_id": "12345"
        }
      }
    ],
    "extension_id": "chrome-extension-id"
  }
}
```

レスポンス:
```json
{
  "success": true,
  "data": {
    "success_count": 3,
    "error_count": 1,
    "errors": ["Error message"],
    "synced_games": ["Game 1", "Game 2", "Game 3"]
  }
}
```

#### get_status
同期状態の取得

レスポンス:
```json
{
  "success": true,
  "data": {
    "last_sync": "2025-01-30T12:34:56Z",
    "total_synced": 42,
    "connected_extensions": ["extension-id"],
    "is_running": true
  }
}
```

#### set_config
設定の更新

リクエスト:
```json
{
  "type": "set_config",
  "payload": {
    "auto_sync": true,
    "allowed_domains": ["games.dmm.co.jp"],
    "sync_interval_minutes": 5,
    "debug_mode": false
  }
}
```

#### health_check
接続確認

レスポンス:
```json
{
  "success": true,
  "data": "OK"
}
```

## セキュリティ

### メッセージサイズ制限
- 最大メッセージサイズ: 1MB
- 超過した場合はエラーを返す

### 拡張機能ID検証
- `allowed_origins`に登録された拡張機能のみ通信可能
- マニフェストファイルで制御

### ログ出力
- すべてのログは標準エラー出力（stderr）に出力
- 標準出力（stdout）は通信専用

## トラブルシューティング

### エラー: "Native host has exited"
- Native Messaging Hostがクラッシュしている
- stderr出力を確認してエラーメッセージを確認

### エラー: "Specified native messaging host not found"
- レジストリ登録が正しく行われていない
- インストールスクリプトを再実行

### エラー: "Access to the specified native messaging host is forbidden"
- 拡張機能IDが`allowed_origins`に含まれていない
- マニフェストファイルを更新して再インストール

## 開発時の注意事項

1. **標準出力の使用制限**
   - `println!`や`print!`を使用しない
   - デバッグ出力は`log::info!`等を使用してstderrに出力

2. **エラーハンドリング**
   - パニックを避け、適切にエラーレスポンスを返す
   - 予期しない入力に対して堅牢に動作する

3. **メモリ管理**
   - 大きなメッセージに対するメモリ使用量に注意
   - 適切なサイズ制限を設ける

4. **PowerShellスクリプトの埋め込み**
   - インストール用のPowerShellスクリプトは`src/usecase/extension_installer.rs`に文字列として埋め込まれています
   - 外部ファイルへの依存を避けるため、スクリプトは実行時に一時ファイルに書き出されて実行されます

## 現在の実装状態

現在のNative Messaging Hostは固定値を返すモック実装です。以下の機能が実装されています：

- ✅ Native Messaging Protocol準拠の通信
- ✅ 4つのメッセージタイプ（sync_games, get_status, set_config, health_check）のハンドリング
- ✅ エラーハンドリングとログ出力
- ✅ セキュリティチェック（メッセージサイズ制限）

今後の実装予定：
- ⏳ Launchergデータベースとの実際の連携
- ⏳ 拡張機能ID検証
- ⏳ レート制限
- ⏳ 詳細な同期ロジック
