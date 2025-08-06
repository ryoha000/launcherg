// Chrome API モックのセットアップ
import { setupChromeMocks } from './mocks/chrome-api'

// グローバルな Chrome API モックを設定
setupChromeMocks()

// その他のグローバル設定
Object.defineProperty(window, 'chrome', {
  value: (globalThis as any).chrome,
  writable: true,
})
