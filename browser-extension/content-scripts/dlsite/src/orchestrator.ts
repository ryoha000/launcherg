import type { DlsiteGame, DlsiteSyncGamesRequest, ExtensionRequest } from '@launcherg/shared'
import type { DlsiteExtractedGame } from './types'
import { generateRequestId, sendExtensionRequest } from '@launcherg/shared'

export async function syncDlsiteGames(games: DlsiteExtractedGame[]): Promise<void> {
  if (games.length === 0)
    return

  const dlsiteGames: DlsiteGame[] = games.map(game => ({
    id: game.storeId,
    category: game.category,
    title: game.title,
    imageUrl: game.imageUrl,
  }))

  const request: ExtensionRequest = {
    requestId: generateRequestId(),
    request: {
      case: 'syncDlsiteGames',
      value: { games: dlsiteGames } as DlsiteSyncGamesRequest,
    },
  }

  await sendExtensionRequest(request)
}
