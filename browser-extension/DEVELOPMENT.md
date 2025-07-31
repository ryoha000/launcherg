# 開発者ガイド - Launcherg DL Store Sync Extension

このドキュメントは、Launcherg DL Store Sync拡張機能の開発・メンテナンス・拡張に関する詳細なガイドです。

## 📋 目次

- [アーキテクチャ概要](#アーキテクチャ概要)
- [開発環境構築](#開発環境構築)
- [コードベース構造](#コードベース構造)
- [抽出システムの詳細](#抽出システムの詳細)
- [新サイト対応手順](#新サイト対応手順)
- [デバッグとテスト](#デバッグとテスト)
- [ビルドとデプロイ](#ビルドとデプロイ)
- [API仕様](#API仕様)

## 🏗️ アーキテクチャ概要

### システム全体図

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Browser   │    │   Browser Ext   │    │   Launcherg     │
│                 │    │                 │    │   Desktop App   │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│  DMM Games      │◄──►│ Content Script  │    │                 │
│  DLsite         │    │  - dmm-ext.ts   │    │                 │
│  (Target Sites) │    │  - dlsite-ext.ts│    │                 │
├─────────────────┤    ├─────────────────┤    │                 │
│                 │    │ Background      │◄──►│ Native Msg Host │
│                 │    │  - bg.ts        │    │  - host.rs      │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│  Popup UI       │◄──►│ Popup           │    │  SQLite DB      │
│  (Extension)    │    │  - popup.ts     │    │  Collection     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### データフロー

1. **検出フェーズ**
   ```
   Page Load → Content Script → Page Detection → Rule Matching
   ```

2. **抽出フェーズ**
   ```
   Rule Matching → DOM Query → Data Extract → Data Normalize
   ```

3. **同期フェーズ**
   ```
   Data Normalize → Background → Native Messaging → Launcherg → Database
   ```

## 🛠️ 開発環境構築

### 必要なツール

```json
{
  "node": ">=16.0.0",
  "npm": ">=7.0.0",
  "chrome": ">=88.0.0",
  "launcherg": ">=1.0.0"
}
```

### セットアップ手順

1. **リポジトリクローン**
```bash
git clone https://github.com/your-repo/launcherg.git
cd launcherg/browser-extension
```

2. **依存関係インストール**
```bash
npm install
```

3. **環境変数設定**
```bash
# .env.development
NODE_ENV=development
DEBUG_MODE=true
NATIVE_HOST_NAME=moe.ryoha.launcherg.extension_host
```

4. **開発サーバー起動**
```bash
npm run dev
# → dist/ フォルダが生成され、ファイル変更を監視
```

5. **Chrome拡張機能として読み込み**
```
1. chrome://extensions/ を開く
2. デベロッパーモードを有効化
3. 「パッケージ化されていない拡張機能を読み込む」
4. dist/ フォルダを選択
```

### 開発用スクリプト

```bash
# 開発モード（ファイル監視）
npm run dev

# 本番ビルド
npm run build

# 型チェック
npm run type-check

# リント
npm run lint

# フォーマット
npm run format

# クリーンビルド
npm run clean && npm run build
```

## 📁 コードベース構造

### ディレクトリ構造詳細

```
src/
├── content-scripts/           # ページ上で実行されるスクリプト
│   ├── base-extractor.ts      # 抽出システムの基底クラス
│   ├── dmm-extractor.ts       # DMM Games専用実装
│   ├── dlsite-extractor.ts    # DLsite専用実装
│   └── site-detector.ts       # サイト検出ユーティリティ
├── background/                # バックグラウンド処理
│   ├── background.ts          # メインのサービスワーカー
│   ├── native-messaging.ts    # Native Messaging制御
│   ├── storage-manager.ts     # ストレージ管理
│   └── sync-scheduler.ts      # 同期スケジューラー
├── popup/                     # 拡張機能UI
│   ├── popup.html            # メインUI
│   ├── popup.ts              # UI制御ロジック
│   ├── styles.css            # スタイルシート
│   └── components/           # UIコンポーネント
│       ├── status-display.ts
│       ├── settings-panel.ts
│       └── log-viewer.ts
├── config/                    # 設定ファイル
│   ├── extraction-rules.json # サイト別抽出ルール
│   ├── default-config.json   # デフォルト設定
│   └── site-configs/         # サイト別詳細設定
│       ├── dmm.json
│       └── dlsite.json
├── types/                     # TypeScript型定義
│   ├── extraction.ts         # 抽出関連の型
│   ├── messaging.ts          # メッセージング関連の型
│   └── config.ts             # 設定関連の型
└── utils/                     # ユーティリティ
    ├── dom-utils.ts          # DOM操作ヘルパー
    ├── date-utils.ts         # 日付処理
    ├── url-utils.ts          # URL処理
    └── logger.ts             # ログシステム
```

### 主要クラス・インターフェース

#### BaseExtractor

```typescript
abstract class BaseExtractor {
  protected config: SiteConfig
  protected debugMode: boolean

  constructor(config: SiteConfig, debugMode?: boolean)

  // 抽象メソッド
  abstract shouldExtract(): boolean
  abstract extractAndSync(): Promise<void>

  // 共通メソッド
  protected detectPage(): boolean
  protected extractGames(): ExtractedGameData[]
  protected extractSingleGame(container: HTMLElement): ExtractedGameData | null
  protected extractField(container: HTMLElement, rule: ExtractionRule): string | null
}
```

#### SiteConfig Interface

```typescript
interface SiteConfig {
  name: string
  domain: string
  detectionRules: ExtractionRule[]
  gameExtractionRules: {
    container: string
    fields: Record<string, ExtractionRule>
  }
  customProcessors?: Record<string, Function>
  waitConditions?: WaitCondition[]
}

interface ExtractionRule {
  name: string
  selectors?: string[]
  selector?: string
  attribute?: string
  fallbackAttribute?: string
  processor?: 'text' | 'html' | 'attr' | 'custom'
  customProcessor?: string
  urlPattern?: string
  required: boolean
  description?: string
  validator?: string // 正規表現またはカスタム関数名
}
```

## 🔍 抽出システムの詳細

### 抽出プロセス

1. **ページ検出**
   ```typescript
   detectPage(): boolean {
     for (const rule of this.config.detectionRules) {
       const element = this.findElement(rule);
       if (rule.required && !element) return false;
       if (element) return true;
     }
     return false;
   }
   ```

2. **ゲームコンテナ検出**
   ```typescript
   const containers = document.querySelectorAll(this.config.gameExtractionRules.container)
   ```

3. **フィールド抽出**
   ```typescript
   extractField(container: HTMLElement, rule: ExtractionRule): string | null {
     const element = this.findElement(rule, container);
     if (!element) return null;

     switch (rule.processor) {
       case 'text': return element.textContent?.trim() || null;
       case 'html': return element.innerHTML;
       case 'attr': return element.getAttribute(rule.attribute!) || null;
       case 'custom': return this.executeCustomProcessor(rule.customProcessor!, element);
       default: return element.textContent?.trim() || null;
     }
   }
   ```

### カスタムプロセッサーシステム

サイト固有の複雑な抽出ロジックに対応：

```typescript
// config/site-configs/dmm.json
{
  "customProcessors": {
    "extractGameId": "function(element) { return element.href.match(/game\\/([^/]+)/)?.[1] || null; }",
    "normalizeDate": "function(element) { return new Date(element.textContent).toISOString(); }"
  }
}

// 実行時
executeCustomProcessor(processorCode: string, element: HTMLElement): string | null {
  try {
    const processor = new Function('element', `return (${processorCode})(element);`);
    return processor(element);
  } catch (error) {
    this.debug('Custom processor error:', error);
    return null;
  }
}
```

### エラーハンドリング

```typescript
interface ExtractionError {
  type: 'DETECTION_FAILED' | 'EXTRACTION_FAILED' | 'VALIDATION_FAILED'
  message: string
  element?: HTMLElement
  rule?: ExtractionRule
  originalError?: Error
}

class ExtractionResult {
  success: boolean
  data: ExtractedGameData[]
  errors: ExtractionError[]
  warnings: string[]

  addError(error: ExtractionError): void
  addWarning(message: string): void
  hasErrors(): boolean
}
```

## 🌐 新サイト対応手順

### 1. サイト分析

1. **ページ構造調査**
   ```bash
   # 対象ページにアクセス
   # F12 → Elements → ゲーム一覧部分を調査
   ```

2. **データ取得パターン特定**
   ```javascript
   // コンソールで実行
   document.querySelectorAll('.game-item').forEach((item, index) => {
     console.log(`Game ${index}:`, {
       title: item.querySelector('.title')?.textContent,
       id: item.dataset.gameId || item.querySelector('a')?.href,
       thumbnail: item.querySelector('img')?.src
     })
   })
   ```

### 2. 設定ファイル作成

1. **基本設定追加** (`src/config/extraction-rules.json`)
   ```json
   {
     "sites": {
       "new_site": {
         "name": "新サイト名",
         "domain": "example.com",
         "detectionRules": [
           {
             "name": "library_detection",
             "selector": ".library-container",
             "required": true,
             "description": "購入済み一覧ページの検出"
           }
         ],
         "gameExtractionRules": {
           "container": ".game-item",
           "fields": {
             "store_id": {
               "selectors": ["[data-game-id]", ".game-link"],
               "attribute": "data-game-id",
               "urlPattern": "/game/([^/]+)",
               "required": true
             },
             "title": {
               "selectors": [".game-title", "h3"],
               "processor": "text",
               "required": true
             }
           }
         }
       }
     }
   }
   ```

2. **詳細設定作成** (`src/config/site-configs/newsite.json`)
   ```json
   {
     "waitConditions": [
       {
         "type": "element",
         "selector": ".game-list",
         "timeout": 5000
       },
       {
         "type": "network",
         "urlPattern": "/api/games",
         "timeout": 3000
       }
     ],
     "customProcessors": {
       "extractId": "function(el) { return el.href.split('/').pop(); }",
       "cleanTitle": "function(el) { return el.textContent.replace(/[\\[\\]]/g, ''); }"
     },
     "dataTransformers": {
       "store_id": "extractId",
       "title": "cleanTitle"
     }
   }
   ```

### 3. Content Script実装

1. **エクストラクター作成** (`src/content-scripts/newsite-extractor.ts`)
   ```typescript
   import { BaseExtractor, ExtractedGameData, SiteConfig } from './base-extractor'

   classNewSiteExtractor extends BaseExtractor {
     constructor(config: SiteConfig) {
       super(config, true))
     }

     shouldExtract(): boolean {
       return window.location.hostname.includes('example.com') && this.detectPage();)
     }
     async extractAndSync(): Promise<void> {
       if (this.isExtracting) ret
   urn;
       this.isExtracting = trutrue

       try      await this.waitForPageLoad();
   )
         constes = this.extractGames();
   )
         constcessedGames = games.map(game => this.processNewSiteGame(game));

         await this.sendToBackground('NewSite', processedGames);
   )
       }ch (error) {
         this.handleError(error);
   )
       }ally {
         this.isExtracting = falsfalse
       }}

     private processNewSiteGame(game: ExtractedGameData): ExtractedGameData {
       // サイト固有の処理
       if (game.purchase_url && !game.purchase_url.startsWith('http')) {
         game.purchase_url = 'http`https://example.com${  game.purchase_url}`  }
 game.additional_data.store_name = 'New 'New Site'
       gametional_data.extraction_source = 'news'newsite-extractor'

       return;game
     }```

2. **Manifest更新** (`manifest.json`)
   ```json
   {
     "content_scripts": [
       {
         "matches": ["https://example.com/*"],
         "js": ["content-scripts/newsite-extractor.js"],
         "run_at": "document_idle"
       }
     ]
   }
   ```

### 4. テストとデバッグ

1. **単体テスト**
   ```bash
   npm run test -- --grep "NewSiteExtractor"
   ```

2. **統合テスト**
   ```bash
   # テストページで確認
   # 1. example.com/library にアクセス
   # 2. F12 → Console → "[NewSite Extractor]" でフィルタ
   # 3. 抽出結果を確認
   ```

3. **E2Eテスト**
   ```bash
   npm run test:e2e -- --site=newsite
   ```

## 🐛 デバッグとテスト

### デバッグツール

1. **内蔵ログシステム**
   ```typescript
   // src/utils/logger.ts
   class Logger {
     static debug(component: string, message: string, ...args: any[]): void {
       if (DEBUG_MODE) {
         console.log(`[${component}] ${message}`, ...args)
       }
     }

     static error(component: string, error: Error, context?: any): void {
       console.error(`[${component}] Error:`, error, context)
     }
   }

   // 使用例
   Logger.debug('DMM Extractor', 'Found games:', games.length))
   Loggererror('Background', new Error('Sync failed'), { gameCount: 5 }))
   ```

2. **抽出結果検証**
   ```typescript
   class ExtractionValidator {
     static validateGameData(data: ExtractedGameData): ValidationResult {
       const errors: string[] = []

       if (!data.store_id)
         errors.push('store_id is required')
       if (!data.title)
         errors.push('title is required')
       if (!data.purchase_url)
         errors.push('purchase_url is required')

       if (data.purchase_url && !this.isValidUrl(data.purchase_url)) {
         errors.push('purchase_url is not a valid URL')
       }

       return { valid: errors.length === 0, errors }
     }
   }
   ```

### テストフレームワーク

1. **Jest設定** (`jest.config.js`)
   ```javascript
   module.exports = {
     preset: 'ts-jest',
     testEnvironment: 'jsdom',
     setupFiles: ['<rootDir>/test/setup.ts'],
     testMatch: ['**/__tests__/**/*.test.ts'],
     collectCoverageFrom: [
       'src/**/*.ts',
       '!src/**/*.d.ts'
     ]
   }
   ```

2. **テストヘルパー** (`test/helpers.ts`)
   ```typescript
   export class MockDOM {
     static createGameList(games: Array<{ title: string, id: string }>): HTMLElement {
       const container = document.createElement('div')
       container.className = 'game-list'

       games.forEach((game) => {
         const item = document.createElement('div')
         item.className = 'game-item'
         item.dataset.gameId = game.id

         const title = document.createElement('h3')
         title.textContent = game.title
         item.appendChild(title)

         container.appendChild(item)
       })

       returncontainer
     }
   }
   ```

3. **抽出テスト例**
   ```typescript
   describe('DMM Extractor', () => {
     let extractor: DMMExtractor

     beforeEach(() => {
       const config = loadSiteConfig('dmm')
       extractor = new DMMExtractor(config)
       document.body.innerHTML = ''
     })

     test'should extract games from mock DOM', () => {
       const mockGames = MockDOM.createGameList([
         { title: 'Test Game 1', id: 'game1' },
         { title: 'Test Game 2', id: 'game2' }
       ])
       document.body.appendChild(mockGames)

       const result = extractor.extractGames()

       expect(result).toHaveLength(2)
       expect(result[0].title).toBe('Test Game 1')
       expect(result[0].store_id).toBe('game1')
     }))
   }
   ```

## 🚀 ビルドとデプロイ

### ビルドプロセス

1. **開発ビルド**
   ```bash
   npm run build:dev
   # → ソースマップ付き、圧縮なし
   ```

2. **本番ビルド**
   ```bash
   npm run build:prod
   # → 最適化、圧縮、難読化
   ```

3. **ビルド設定** (`webpack.config.js`)
   ```javascript
   const config = {
     mode: process.env.NODE_ENV || 'development',
     entry: {
       'background/background': './src/background/background.ts',
       'content-scripts/dmm-extractor': './src/content-scripts/dmm-extractor.ts',
       'popup/popup': './src/popup/popup.ts'
     },
     optimization: {
       minimize: process.env.NODE_ENV === 'production',
       splitChunks: {
         chunks: 'all',
         cacheGroups: {
           vendor: {
             test: /[\\/]node_modules[\\/]/,
             name: 'vendors',
             chunks: 'all'
           }
         }
       }
     }
   }
   ```

### Chrome Web Store公開

1. **マニフェスト準備**
   ```json
   {
     "name": "Launcherg DL Store Sync",
     "version": "1.0.0",
     "description": "Sync your purchased games from DL stores to Launcherg",
     "permissions": ["nativeMessaging", "activeTab", "storage"],
     "host_permissions": [
       "https://games.dmm.co.jp/*",
       "https://www.dlsite.com/*"
     ]
   }
   ```

2. **アセット準備**
   ```bash
   # アイコン作成（複数サイズ）
   icons/
   ├── icon16.png
   ├── icon32.png
   ├── icon48.png
   └── icon128.png

   # スクリーンショット
   screenshots/
   ├── popup.png
   ├── dmm-sync.png
   └── settings.png
   ```

3. **公開用パッケージ作成**
   ```bash
   npm run package
   # → extension.zip 生成
   ```

## 📡 API仕様

### Native Messaging Protocol

#### メッセージ形式

```typescript
interface NativeMessage<T = any> {
  type: MessageType
  payload: T
  timestamp: string
  request_id: string
}

type MessageType = 'sync_games' | 'get_status' | 'set_config' | 'health_check'
```

#### 同期リクエスト

```typescript
interface SyncGamesRequest {
  store: 'DMM' | 'DLSite'
  games: ExtractedGameData[]
  extension_id: string
}

interface SyncGamesResponse {
  success_count: number
  error_count: number
  errors: string[]
  synced_games: string[]
}
```

### Chrome Extension API使用

1. **Runtime Messaging**
   ```typescript
   // Content Script → Background
   chrome.runtime.sendMessage({
     type: 'sync_games',
     store: 'DMM',
     games: extractedGames
   })

   // Background → Content Script
   chrome.tabs.sendMessage(tabId, {
     type: 'manual_sync_request'
   })
   ```

2. **Storage API**
   ```typescript
   // 設定保存
   chrome.storage.local.set({
     extension_config: config
   })

   // 設定読み込み
   const result = await chrome.storage.local.get(['extension_config'])
   ```

3. **Notifications API**
   ```typescript
   chrome.notifications.create({
     type: 'basic',
     iconUrl: 'icons/icon32.png',
     title: 'Launcherg Sync',
     message: '5個のゲームを同期しました'
   })
   ```

---

このドキュメントは継続的に更新されます。質問や提案がありましたら、GitHubのIssuesでお知らせください。
