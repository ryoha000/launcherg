# Event Listener Composables

型安全なTauriイベントリスナーを提供するシンプルなライブラリです。

## 基本的な使い方

### useEvent - メインのcomposable

ほとんどの用途に対応できるシンプルで型安全なイベントリスナーです。

```typescript
import { useEvent } from '$lib/event'

const event = useEvent()

// 型安全なイベントリスニング
await event.startListen('progress', (payload) => {
  // payload は自動的に ProgressPayload 型
  console.log(payload.message)
})

await event.startListen('progresslive', (payload) => {
  // payload は自動的に ProgressLivePayload 型
  console.log(payload.max)
})

// 特定のイベントを停止
event.stopListen('progress')

// すべてのイベントを停止
event.stopAll()
```

### useProgressListener - 進捗専用composable

進捗管理に特化した便利機能付きのcomposableです。

```typescript
import { useProgressListener } from '$lib/event'

const progress = useProgressListener()

await progress.startListen()

// リアクティブな状態にアクセス
$effect(() => {
  console.log(`進捗: ${progress.processedFiles()}/${progress.totalFiles()}`)
  console.log(`進捗率: ${progress.progressPercentage()}%`)
  console.log(`メッセージ: ${progress.currentMessage()}`)
})

progress.stopListen()
progress.resetProgress()
```

## 型定義

サポートされているイベント:

```typescript
export interface EventPayloadMap {
  progress: ProgressPayload // { message: string }
  progresslive: ProgressLivePayload // { max: number | null }
}
```

新しいイベント型を追加する場合は、`types.ts`の`EventPayloadMap`にエントリを追加してください。

## 特徴

- 🔒 **型安全**: コンパイル時に型チェック
- 🎯 **シンプル**: メインは`useEvent`一つだけ
- 🔄 **リアクティブ**: Svelte 5の$stateと統合
- 🧹 **自動管理**: リスナーの適切な停止処理

## ファイル構成

```
src/lib/event/
├── useEvent.svelte.ts           # メインのcomposable
├── useProgressListener.svelte.ts # 進捗専用composable
├── types.ts                     # 型定義
└── index.ts                     # エクスポート
```
