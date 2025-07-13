# Event Listener Composable

型安全なTauriイベントリスナーを提供するシンプルなライブラリです。

## 基本的な使い方

### useEvent - 型安全なイベントリスナー

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
  if (payload.max) {
    console.log(`総ファイル数: ${payload.max}`)
  }
  else {
    console.log('ファイル処理完了')
  }
})

// 特定のイベントを停止
event.stopListen('progress')

// すべてのイベントを停止
event.stopAll()
```

## 実装例：進捗管理

```typescript
import { useEvent } from '$lib/event'

export function useImportProgress() {
  let totalFiles = $state(0)
  let processedFiles = $state(0)
  let currentMessage = $state('')

  const event = useEvent()

  const startListening = async () => {
    await event.startListen('progresslive', (payload) => {
      if (payload.max) {
        totalFiles = payload.max
      }
      else {
        processedFiles++
      }
    })

    await event.startListen('progress', (payload) => {
      currentMessage = payload.message
    })
  }

  const stopListening = () => {
    event.stopAll()
  }

  const progressPercentage = () => {
    if (totalFiles === 0)
      return 0
    return Math.round((processedFiles / totalFiles) * 100)
  }

  return {
    totalFiles: () => totalFiles,
    processedFiles: () => processedFiles,
    currentMessage: () => currentMessage,
    progressPercentage,
    startListening,
    stopListening,
  }
}
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
- 🎯 **シンプル**: 単一のAPIですべてをカバー
- 🔄 **リアクティブ**: Svelte 5の$stateと統合
- 🧹 **自動管理**: リスナーの適切な停止処理

## ファイル構成

```
src/lib/event/
├── useEvent.svelte.ts  # メインのcomposable
├── types.ts           # 型定義
└── index.ts          # エクスポート
```
