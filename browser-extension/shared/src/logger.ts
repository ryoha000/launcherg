// 共有ロガー（名前空間 + ログレベル）

export type LogLevel = 'silent' | 'error' | 'warn' | 'info' | 'debug'

const levelOrder = { silent: 0, error: 1, warn: 2, info: 3, debug: 4 } as const

let currentLevel: LogLevel
  = (typeof localStorage !== 'undefined'
    ? (localStorage.getItem('launcherg:log-level') as LogLevel)
    : undefined) || 'info'

function shouldLog(target: keyof typeof levelOrder): boolean {
  return levelOrder[target] <= levelOrder[currentLevel]
}

export function setLogLevel(level: LogLevel): void {
  currentLevel = level
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('launcherg:log-level', level)
    }
  }
  catch {
    // ignore
  }
}

export function getLogLevel(): LogLevel {
  return currentLevel
}

export function logger(namespace: string) {
  const prefix = `[${namespace}]`
  return {
    debug: (...args: any[]) => shouldLog('debug') && console.log(prefix, ...args),
    info: (...args: any[]) => shouldLog('info') && console.log(prefix, ...args),
    warn: (...args: any[]) => shouldLog('warn') && console.warn(prefix, ...args),
    error: (...args: any[]) => shouldLog('error') && console.error(prefix, ...args),
  }
}
