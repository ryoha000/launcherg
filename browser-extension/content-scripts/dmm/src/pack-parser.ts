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

// Find a child game image inside pack modal (#js-detail) that matches given storeId
export function findChildGameImage(root: Document | HTMLElement, storeId: string): HTMLImageElement | null {
  const scope = (root instanceof Document)
    ? (root.getElementById('js-detail') ?? root)
    : root

  const images = Array.from(scope.querySelectorAll('img')) as HTMLImageElement[]
  for (const img of images) {
    const game = extractGameDataFromImage(img)
    if (game && game.storeId === storeId)
      return img
  }
  return null
}
