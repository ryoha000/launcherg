import type { DmmExtractedGame } from './types'
import { extractGameDataFromImage } from './dom-extractor'

export function parsePackModal(html: string): DmmExtractedGame[] {
  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')
  const root = doc.getElementById('js-detail') || doc
  const result: DmmExtractedGame[] = []

  const images = Array.from(root.querySelectorAll('img'))
  for (const img of images) {
    const game = extractGameDataFromImage(img)
    if (!game)
      continue
    result.push(game)
  }

  return result
}
