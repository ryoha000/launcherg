# Event Listener Composables

型安全なTauriイベントリスナーを提供するcomposableライブラリです。

## 使用可能なComposables

### 1. useEventListener
単一イベント用のシンプルなリスナー

```typescript
import { useEventListener } from '$lib/event'

const eventListener = useEventListener()

// 型安全なイベントリスニング
await eventListener.startListen('progress', (payload) => {
  // payload は自動的に ProgressPayload 型
  console.log(payload.message)
})

eventListener.stopListen()
```

### 2. useTypedEventListener
より強い型制約を持つEventListenerのラッパー

```typescript
import { useTypedEventListener } from '$lib/event'

// イベント名を固定することでより型安全に
const progressListener = useTypedEventListener('progress')

await progressListener.startListen((payload) => {
  // payload は自動的に ProgressPayload 型になる
  console.log(payload.message)
})
```

### 3. useMultiEventListener
複数イベントを同時に管理

```typescript
import { useMultiEventListener } from '$lib/event'

const eventListener = useMultiEventListener()

// 個別にリスナーを追加
await eventListener.startListen('progress', (payload) => {
  console.log(payload.message)
})
await eventListener.startListen('progresslive', (payload) => {
  console.log(payload.max)
})

// または一括で追加
await eventListener.startMultipleListen([
  {
    eventName: 'progress',
    handler: payload => console.log(payload.message)
  },
  {
    eventName: 'progresslive',
    handler: payload => console.log(payload.max)
  }
])

// 特定のイベントを停止
eventListener.stopListen('progress')

// すべて停止
eventListener.stopAllListeners()
```

### 4. useProgressListener
progress/progressliveイベント専用の高機能リスナー

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

サポートされているイベントは `types.ts` で定義されています：

```typescript
export interface EventPayloadMap {
  progress: ProgressPayload // { message: string }
  progresslive: ProgressLivePayload // { max: number | null }
}
```

新しいイベント型を追加する場合は、`EventPayloadMap` にエントリを追加してください。

## 特徴

- 🔒 **型安全**: TypeScriptによる完全な型安全性
- 🎯 **柔軟性**: 単一イベントから複数イベントまで対応
- 🔄 **リアクティブ**: Svelte 5の$stateと統合
- 🧹 **自動クリーンアップ**: リスナーの適切な停止処理
- 📊 **進捗管理**: 進捗率計算などの便利機能
