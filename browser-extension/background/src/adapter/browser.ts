import type { Browser } from '../shared/types'

export function createBrowser(): Browser {
  return {
    alarms: {
      create: async (name: string, options: { when?: number, delayInMinutes?: number, periodInMinutes?: number }) => {
        const alarmInfo: chrome.alarms.AlarmCreateInfo = {}
        if (typeof options?.when === 'number')
          alarmInfo.when = options.when
        if (typeof options?.delayInMinutes === 'number')
          alarmInfo.delayInMinutes = options.delayInMinutes
        if (typeof options?.periodInMinutes === 'number')
          alarmInfo.periodInMinutes = options.periodInMinutes
        chrome.alarms.create(name, alarmInfo)
      },
    },
    notifications: {
      create: async (options) => {
        const chromeOptions: chrome.notifications.NotificationOptions<true> = {
          type: 'basic',
          iconUrl: options.iconUrl,
          title: options.title,
          message: options.message,
        }
        await new Promise<void>((resolve, reject) => {
          chrome.notifications.create(chromeOptions, () => {
            const err = chrome.runtime?.lastError
            if (err) {
              reject(new Error(err.message))
              return
            }
            resolve()
          })
        })
      },
    },
    runtime: {
      getURL: path => chrome.runtime.getURL(path),
    },
    storage: {
      get: keys => new Promise((resolve, reject) => {
        chrome.storage.local.get(keys, (items) => {
          const err = chrome.runtime?.lastError
          if (err) {
            reject(new Error(err.message))
            return
          }
          resolve(items)
        })
      }),
      set: items => new Promise((resolve, reject) => {
        chrome.storage.local.set(items, () => {
          const err = chrome.runtime?.lastError
          if (err) {
            reject(new Error(err.message))
            return
          }
          resolve()
        })
      }),
    },
    tabs: {
      query: queryInfo => new Promise((resolve, reject) => {
        chrome.tabs.query(queryInfo, (tabs) => {
          const err = chrome.runtime?.lastError
          if (err) {
            reject(new Error(err.message))
            return
          }
          resolve(tabs.map(t => ({ id: t.id, url: t.url })))
        })
      }),
      sendMessage: (tabId, message) => new Promise<void>((resolve, reject) => {
        chrome.tabs.sendMessage(tabId, message, () => {
          const err = chrome.runtime?.lastError
          if (err) {
            reject(new Error(err.message))
            return
          }
          resolve()
        })
      }),
    },
    scripting: {
      executeScript: async (tabId, files) => {
        await new Promise<void>((resolve, reject) => {
          chrome.scripting.executeScript({ target: { tabId }, files }, () => {
            const err = chrome.runtime?.lastError
            if (err) {
              reject(new Error(err.message))
              return
            }
            resolve()
          })
        })
      },
    },
  }
}
