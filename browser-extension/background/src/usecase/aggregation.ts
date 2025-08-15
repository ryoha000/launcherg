const AGGREGATE_ALARM = 'notify_aggregate'
const AGGREGATE_COUNT_KEY = 'aggregate_sync_count'

export { AGGREGATE_ALARM }

export async function recordSyncAggregation(count: number): Promise<void> {
  const current = await getAggregateCount()
  await setAggregateCount(current + count)
  chrome.alarms.create(AGGREGATE_ALARM, {
    when: Date.now() + 30_000,
  })
}

export async function fireAggregateNotification(): Promise<void> {
  const total = await getAggregateCount()
  if (total > 0) {
    const title = 'Launcherg DL Store Sync'
    const message = `過去30秒間に合計${total}件を同期しました`
    await chrome.notifications.create({
      type: 'basic',
      iconUrl: chrome.runtime.getURL('icons/icon32.png'),
      title,
      message,
    })
    await setAggregateCount(0)
  }
}

async function getAggregateCount(): Promise<number> {
  return new Promise((resolve) => {
    chrome.storage.local.get([AGGREGATE_COUNT_KEY], (items) => {
      const value = items[AGGREGATE_COUNT_KEY]
      resolve(typeof value === 'number' ? value : 0)
    })
  })
}

async function setAggregateCount(value: number): Promise<void> {
  return new Promise((resolve) => {
    chrome.storage.local.set({ [AGGREGATE_COUNT_KEY]: value }, () => resolve())
  })
}
