/**
 * Work中心スキーマ移行（V3）: ローカル永続データのマイグレーション
 * - tabs: workId を数値から文字列へ統一（UUID対応）
 * - memos: キー形式は維持（正規表現の緩和は呼び出し側で実施）
 */

const MIGRATION_FLAG_KEY = 'migration.work-centered.v3'

export async function runLocalMigrationV3(): Promise<void> {
  // 既に実行済みならスキップ
  if (localStorage.getItem(MIGRATION_FLAG_KEY)) {
    return
  }

  try {
    // tabs のマイグレーション
    const tabsRaw = localStorage.getItem('tabs')
    if (tabsRaw) {
      try {
        const tabs = JSON.parse(tabsRaw) as unknown[]
        if (Array.isArray(tabs)) {
          const migratedTabs = tabs
            .filter((t): t is Record<string, unknown> => {
              // null/undefined やオブジェクトでないものを除外
              if (!t || typeof t !== 'object') {
                return false
              }
              // タブの基本構造を持っているかチェック（id, type, scrollTo, title が存在）
              const tab = t as Record<string, unknown>
              return typeof tab.id === 'number'
                && typeof tab.type === 'string'
                && typeof tab.scrollTo === 'number'
                && typeof tab.title === 'string'
            })
            .map((tab) => {
              // workId を文字列に統一（数値なら文字列化）
              const workId = tab.workId
              const workIdStr
                = typeof workId === 'number'
                  ? String(workId)
                  : typeof workId === 'string'
                    ? workId
                    : workId != null
                      ? String(workId)
                      : ''

              return {
                ...tab,
                workId: workIdStr,
              }
            })

          localStorage.setItem('tabs', JSON.stringify(migratedTabs))
        }
      }
      catch (e) {
        // JSON パースエラーや変換エラーは無視（既存データが壊れている可能性）
        console.warn('Failed to migrate tabs:', e)
      }
    }

    // マイグレーション完了フラグを設定
    localStorage.setItem(MIGRATION_FLAG_KEY, String(Date.now()))
  }
  catch (e) {
    // エラーが発生してもフラグを設定して再実行を防ぐ
    console.error('Migration V3 failed:', e)
    localStorage.setItem(MIGRATION_FLAG_KEY, String(Date.now()))
  }
}
