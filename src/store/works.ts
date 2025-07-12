import type { Work } from '@/lib/types'
import { getWorkByScrape } from '@/lib/scrape/scrapeWork'
import { createLocalStorageCache } from '@/lib/utils'

function createWorks() {
  const getter = createLocalStorageCache<number, Work | undefined>(
    'works-cache',
    getWorkByScrape,
  )

  return {
    get: getter,
  }
}

export const works = createWorks()
