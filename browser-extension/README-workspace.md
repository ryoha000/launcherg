# Browser Extension Workspace Setup

このディレクトリは pnpm workspace として構成されています。

## 構造

```
browser-extension/
├── pnpm-workspace.yaml     # Workspace設定
├── package.json            # ルートのpackage.json
├── scripts/
│   └── copy-assets.js      # アセットコピー用スクリプト
├── shared/                 # 共通ライブラリ
│   ├── src/
│   │   ├── proto/         # Protocol Buffers
│   │   ├── types/         # 型定義
│   │   ├── base-extractor.ts  # 共通エクストラクター
│   │   └── index.ts       # エクスポート
│   ├── package.json
│   └── tsconfig.json
├── background/             # Background script
│   ├── src/
│   │   └── background.ts
│   ├── package.json
│   ├── tsconfig.json
│   └── vite.config.ts
├── popup/                  # Popup UI
│   ├── src/
│   │   ├── popup.ts
│   │   ├── popup.html
│   │   └── styles.css
│   ├── package.json
│   ├── tsconfig.json
│   └── vite.config.ts
├── content-scripts/
│   ├── dlsite/             # DLsite content script
│   │   ├── src/
│   │   │   └── dlsite-extractor.ts
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   └── vite.config.ts
│   └── dmm/                # DMM content script
│       ├── src/
│       │   └── dmm-extractor.ts
│       ├── package.json
│       ├── tsconfig.json
│       └── vite.config.ts
└── dist/                   # 最終ビルド結果
    ├── manifest.json
    ├── background/
    ├── popup/
    ├── content-scripts/
    └── config/
```

## ビルド方法

### 全体ビルド
```bash
pnpm run build
```

### 開発モード
```bash
pnpm run dev
```

### クリーンアップ
```bash
pnpm run clean
```

### 個別ワークスペースのビルド
```bash
# 共有ライブラリのビルド
pnpm run --filter @launcherg/shared build

# Background scriptのビルド
pnpm run --filter @launcherg/background build

# Popup UIのビルド
pnpm run --filter @launcherg/popup build

# Content scriptsのビルド
pnpm run --filter @launcherg/content-scripts-dlsite build
pnpm run --filter @launcherg/content-scripts-dmm build
```

## Protocol Buffers

Protocol Buffersファイルは `shared/src/proto/` に配置され、ビルド時に自動生成されます。

## 依存関係

- `@launcherg/shared`: 全てのワークスペースが共通で使用するライブラリ
- 各ワークスペースは `@launcherg/shared` に依存

## 注意事項

1. 共通の型や関数は `shared` パッケージに配置してください
2. 各ワークスペースは独立してビルドできるように設計されています
3. Protocol Buffersの変更時は `shared` パッケージから再ビルドしてください