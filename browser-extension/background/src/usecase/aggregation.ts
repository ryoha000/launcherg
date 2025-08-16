import type { Browser } from '../shared/types'

const AGGREGATE_ALARM = 'notify_aggregate'
const AGGREGATE_COUNT_KEY = 'aggregate_sync_count'

export { AGGREGATE_ALARM }

export async function recordSyncAggregation(browser: Browser, count: number): Promise<void> {
  const current = await getAggregateCount(browser)
  await setAggregateCount(browser, current + count)
  await browser.alarms.create(AGGREGATE_ALARM, {
    when: Date.now() + 30_000,
  } as any)
}

export async function fireAggregateNotification(browser: Browser): Promise<void> {
  const total = await getAggregateCount(browser)
  if (total > 0) {
    const title = 'Launcherg DL Store Sync'
    const message = `過去30秒間に合計${total}件を同期しました`
    await browser.notifications.create({
      type: 'basic',
      iconUrl: browser.runtime.getURL('icons/icon32.png'),
      title,
      message,
    })
    await setAggregateCount(browser, 0)
  }
}

async function getAggregateCount(browser: Browser): Promise<number> {
  const items = await browser.storage.get([AGGREGATE_COUNT_KEY])
  const value = items[AGGREGATE_COUNT_KEY]
  return typeof value === 'number' ? value : 0
}

async function setAggregateCount(browser: Browser, value: number): Promise<void> {
  await browser.storage.set({ [AGGREGATE_COUNT_KEY]: value })
}
