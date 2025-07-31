# Launcherg DL Store Sync 拡張機能 - インストールガイド

DMM Games と DLsite の購入済みゲームを自動的に Launcherg デスクトップアプリに同期する Chrome/Edge 拡張機能です。

## 📋 システム要件

- **OS**: Windows 10/11
- **ブラウザ**: Chrome 88+ または Edge 88+
- **その他**: Launcherg デスクトップアプリ

## 🚀 インストール手順

### 方法1: 自動インストール（推奨）

1. **拡張機能をインストール**
   - Chrome: [Chrome Web Store からインストール](#) (公開予定)
   - または手動インストール（下記参照）

2. **Native Messaging Host を自動インストール**
   ```powershell
   # PowerShellを管理者権限で実行
   cd "C:\Path\To\Launcherg\src-tauri"
   .\auto-install-native-messaging-host.ps1
   ```

   このスクリプトが以下を自動実行します：
   - インストール済み拡張機能のIDを自動検出
   - Native Messaging Host の登録
   - レジストリエントリの作成

3. **動作確認**
   - Launcherg デスクトップアプリを起動
   - DMM Games または DLsite のライブラリページを開く
   - 拡張機能アイコンをクリックして「手動同期」をテスト

### 方法2: 手動インストール

#### 拡張機能の手動インストール

1. **ビルド**
   ```bash
   cd browser-extension
   npm install
   npm run build
   ```

2. **Chrome/Edge にインストール**
   - `chrome://extensions/` または `edge://extensions/` を開く
   - 「デベロッパーモード」を有効化
   - 「パッケージ化されていない拡張機能を読み込む」
   - `browser-extension/dist` フォルダを選択
   - 表示される **Extension ID をメモ**

#### Native Messaging Host の手動インストール

1. **Native Messaging Host をビルド**
   ```bash
   cd src-tauri
   cargo build --release --bin native-messaging-host
   ```

2. **手動登録**
   ```powershell
   # PowerShellを管理者権限で実行
   cd src-tauri
   .\install-native-messaging-host.ps1 -ExtensionId "YOUR_EXTENSION_ID_HERE"
   ```

## 🔧 トラブルシューティング

### 拡張機能が見つからない

```
❌ エラー: Launcherg DL Store Sync 拡張機能が見つかりませんでした
```

**解決方法:**
1. Chrome/Edge で拡張機能が正しくインストールされているか確認
2. 拡張機能名が「Launcherg DL Store Sync」になっているか確認
3. ブラウザを再起動してから再実行
4. 手動インストール方法を使用

### Native Messaging エラー

```
❌ Native messaging timeout / Connection failed
```

**解決方法:**
1. Launcherg デスクトップアプリが起動しているか確認
2. PowerShell を管理者権限で実行したか確認
3. Windows ファイアウォールが通信をブロックしていないか確認
4. 以下のレジストリエントリが存在するか確認：
   - `HKCU\Software\Google\Chrome\NativeMessagingHosts\moe.ryoha.launcherg.extension_host`
   - `HKCU\Software\Microsoft\Edge\NativeMessagingHosts\moe.ryoha.launcherg.extension_host`

### 同期が動作しない

1. **権限確認**
   - DMM Games: `https://games.dmm.co.jp/*`
   - DLsite: `https://www.dlsite.com/*`

2. **デバッグモード有効化**
   - F12 → Console で `[Background]` や `[DMM Extractor]` のログを確認

3. **手動同期テスト**
   - 対象ページで拡張機能アイコンをクリック
   - 「手動同期」ボタンをクリック

## 🔄 アンインストール

1. **拡張機能を削除**
   - `chrome://extensions/` で拡張機能を削除

2. **Native Messaging Host を削除**
   ```powershell
   cd src-tauri
   .\uninstall-native-messaging-host.ps1
   ```

## 📞 サポート

問題が解決しない場合：

1. **ログ収集**
   - ブラウザのコンソールログ（F12 → Console）
   - Launcherg デスクトップアプリのログ

2. **Issue報告**
   - [GitHub Issues](https://github.com/your-repo/launcherg/issues)
   - 収集したログを添付してください

## 🔐 セキュリティについて

- Native Messaging は Chrome/Edge の公式セキュリティメカニズムです
- 通信は拡張機能とデスクトップアプリ間でのみ行われます
- ゲームデータはローカルのSQLiteデータベースにのみ保存されます
- ネットワーク経由でのデータ送信は行いません
