// 通知関連の関数

import { create, toJson } from '@bufbuild/protobuf'
import {
  ExtensionRequestSchema,
  ShowNotificationRequestSchema,
} from '@launcherg/shared/proto/extension_internal'
import { generateRequestId } from './utils'

// ページ内通知を表示する純粋関数（副作用あり）
export function showInPageNotification(
  message: string,
  type: 'success' | 'error',
): void {
  const notification = document.createElement('div')
  notification.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    background: ${type === 'success' ? '#4CAF50' : '#f44336'};
    color: white;
    padding: 12px 20px;
    border-radius: 4px;
    z-index: 10000;
    font-family: Arial, sans-serif;
    font-size: 14px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.3);
    animation: slideIn 0.3s ease-out;
  `

  notification.textContent = message
  document.body.appendChild(notification)

  // 4秒後に自動削除
  setTimeout(() => {
    if (notification.parentNode) {
      notification.parentNode.removeChild(notification)
    }
  }, 4000)
}

// 通知アニメーションスタイルを追加する関数
export function addNotificationStyles(): void {
  const style = document.createElement('style')
  style.textContent = `
    @keyframes slideIn {
      from {
        transform: translateX(100%);
        opacity: 0;
      }
      to {
        transform: translateX(0);
        opacity: 1;
      }
    }
  `
  document.head.appendChild(style)
}

// 通知リクエストを作成する純粋関数
export function createNotificationRequest(
  message: string,
  type: 'success' | 'error',
): any {
  return create(ExtensionRequestSchema, {
    requestId: generateRequestId(),
    request: {
      case: 'showNotification',
      value: create(ShowNotificationRequestSchema, {
        title: 'Launcherg DL Store Sync',
        message,
        iconType: type,
      }),
    },
  })
}

// 通知を表示する関数（ブラウザ通知とページ内通知の両方）
export function showNotification(
  message: string,
  type: 'success' | 'error' = 'success',
): void {
  // プロトバフで通知メッセージを作成
  const notificationRequest = createNotificationRequest(message, type)

  // ブラウザ通知を表示
  chrome.runtime.sendMessage(
    toJson(ExtensionRequestSchema, notificationRequest),
  )

  // ページ内通知も表示
  showInPageNotification(message, type)
}
