# Launcherg Remote Share Server

Cloudflare Workers 上で動く Remote Share 用の API / WebUI です。  
`/api/*` は Workers 側の API、`/*` は `server/ui/` で build した React SPA を返します。

## 構成

- `src/`
  - Workers 本体
  - oRPC contract / schema
  - D1 / R2 / Cookie セッション処理
- `ui/`
  - Vite + React SPA
- `migrations/`
  - D1 schema

## 前提

- Node.js 20 以上
- npm
- Cloudflare アカウント
- `wrangler` にログイン済み

```powershell
cd F:\workspace\launcherg\server
npx wrangler login
```

## セットアップ

### 1. 依存をインストール

```powershell
cd F:\workspace\launcherg\server
npm install
```

### 2. D1 を作成

未作成なら D1 を作ります。

```powershell
npx wrangler d1 create launcherg
```

返ってきた `database_id` を [wrangler.toml](/F:/workspace/launcherg/server/wrangler.toml) の `[[d1_databases]]` に反映します。

### 3. R2 バケットを作成

未作成なら R2 バケットを作ります。

```powershell
npx wrangler r2 bucket create launcherg-images
```

バケット名を [wrangler.toml](/F:/workspace/launcherg/server/wrangler.toml) の `[[r2_buckets]]` に合わせてください。

### 4. シークレットを設定

閲覧セッション署名用の secret を登録します。

```powershell
npx wrangler secret put SESSION_SECRET
```

### 5. D1 マイグレーションを適用

ローカル開発用:

```powershell
npx wrangler d1 execute launcherg --local --file=.\migrations\0001_initial.sql
```

リモート環境用:

```powershell
npx wrangler d1 execute launcherg --remote --file=.\migrations\0001_initial.sql
```

`launcherg` は [wrangler.toml](/F:/workspace/launcherg/server/wrangler.toml) の `database_name` と合わせてください。

## 開発

### 最短で動かす

1 回 UI を build してから Workers を起動します。

```powershell
cd F:\workspace\launcherg\server
npm run build:ui
npm run dev
```

### UI を編集中に追従させる

別ターミナルで UI build watch を回しておくと、Workers 側が `ui/dist` を配信し続けます。

ターミナル 1:

```powershell
cd F:\workspace\launcherg\server
npm run build:ui:watch
```

ターミナル 2:

```powershell
cd F:\workspace\launcherg\server
npm run dev
```

### SPA 単体で確認する

UI だけを Vite dev server で確認したいとき:

```powershell
cd F:\workspace\launcherg\server
npm run dev:ui
```

この場合、`/api` は Workers へ自動では流れません。  
API 動作込みで確認したいときは `npm run dev` を使ってください。

## チェック

```powershell
cd F:\workspace\launcherg\server
npm run check
```

実行内容:

- TypeScript typecheck
- Vitest
- React SPA build

ルート側からは次でも実行できます。

```powershell
cd F:\workspace\launcherg
task check:server
```

## デプロイ

### 1. 本番用 D1 に schema を入れる

```powershell
cd F:\workspace\launcherg\server
npx wrangler d1 execute launcherg --remote --file=.\migrations\0001_initial.sql
```

### 2. secret を設定

```powershell
npx wrangler secret put SESSION_SECRET
```

### 3. デプロイ

```powershell
npm run deploy
```

このコマンドは以下を実行します。

1. `ui/` を build
2. `.dev.vars` から `R2_ACCOUNT_ID` / `R2_ACCESS_KEY_ID` / `R2_SECRET_ACCESS_KEY` を読み込んで Workers を deploy

`.dev.vars` に値がないと deploy は失敗します。  
本番でこの方式を使う場合、`.dev.vars` に置いた値がそのままデプロイ設定の入力元になります。

## Launcherg 側との接続

Tauri アプリの `Settings > Remote Share` で以下を設定します。

- `Server Base URL`
  - 例: `https://launcherg.<your-subdomain>.workers.dev`
- `deviceSecret`
  - ユーザーが決める共有用 secret

その後の流れ:

1. `device を登録`
2. `作品を同期`
3. 生成された共有 URL / QR をスマホで開く
4. 同じ `deviceSecret` を入力して一覧を確認する

画像アップロードは Cloudflare R2 の presigned `PUT` を使います。  
そのため Workers 側には次の設定が必要です。

- `R2_ACCOUNT_ID`
  - Cloudflare の Account ID。`wrangler.toml` の `[vars]` か Dashboard で設定してください。
- `R2_ACCESS_KEY_ID`
- `R2_SECRET_ACCESS_KEY`
- `R2_BUCKET_NAME`
- `R2_PRESIGN_TTL_SECONDS` は任意

## よくある詰まりどころ

### `deviceSecret is invalid`

- Tauri 側に保存されている `deviceSecret` と、Web 側で入力した `deviceSecret` が一致していません。
- `deviceId` を再発行したい場合は、別の secret を使って再登録してください。

### 画像が出ない

- `作品を同期` を再実行してください。
- サムネイルがローカルに存在しない作品は画像なしで同期されます。

### `wrangler deploy` で binding error が出る

- [wrangler.toml](/F:/workspace/launcherg/server/wrangler.toml) の `database_id` / `bucket_name` を実環境に合わせてください。
- `SESSION_SECRET` が登録されているか確認してください。
