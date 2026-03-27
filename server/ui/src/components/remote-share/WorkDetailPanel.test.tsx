import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it } from 'vitest'

import { LaunchResultAlert } from '@ui/components/remote-share/WorkDetailPanel'
import type { LaunchWorkResponse } from '@server/shared/schema'

describe('LaunchResultAlert', () => {
  const cases: Array<{
    name: string
    result: LaunchWorkResponse
    title: string
    message: string
  }> = [
    {
      name: 'launched で成功表示になる',
      result: {
        ok: true,
        reasonCode: 'launched',
        message: 'デスクトップアプリでゲームを起動しました。',
        detail: null,
      },
      title: '起動しました',
      message: 'デスクトップアプリでゲームを起動しました。',
    },
    {
      name: 'desktop_not_connected で専用文言が表示される',
      result: {
        ok: false,
        reasonCode: 'desktop_not_connected',
        message: 'デスクトップアプリが Remote Launch に接続していません。Launcherg を起動した状態で再試行してください。',
        detail: 'desktop websocket is not connected',
      },
      title: '起動できませんでした',
      message: 'デスクトップアプリが Remote Launch に接続していません。Launcherg を起動した状態で再試行してください。',
    },
    {
      name: 'shortcut_not_found で専用文言が表示される',
      result: {
        ok: false,
        reasonCode: 'shortcut_not_found',
        message: 'この作品には起動用ショートカットが登録されていません。PC 側で作品登録状態を確認してください。',
        detail: 'no launchable lnk found',
      },
      title: '起動できませんでした',
      message: 'この作品には起動用ショートカットが登録されていません。PC 側で作品登録状態を確認してください。',
    },
    {
      name: 'target_not_installed で専用文言が表示される',
      result: {
        ok: false,
        reasonCode: 'target_not_installed',
        message: '起動先のファイルが見つかりませんでした。ゲームが未インストールか、インストール先が移動された可能性があります。',
        detail: 'shortcut target path does not exist',
      },
      title: '起動できませんでした',
      message: '起動先のファイルが見つかりませんでした。ゲームが未インストールか、インストール先が移動された可能性があります。',
    },
    {
      name: 'launch_failed で専用文言が表示される',
      result: {
        ok: false,
        reasonCode: 'launch_failed',
        message: 'ゲームの起動に失敗しました。PC 側で直接起動できるか確認してください。',
        detail: 'ShellExecuteExW failed',
      },
      title: '起動できませんでした',
      message: 'ゲームの起動に失敗しました。PC 側で直接起動できるか確認してください。',
    },
  ]

  it.each(cases)('$name', ({ result, title, message }) => {
    const html = renderToStaticMarkup(<LaunchResultAlert result={result} />)

    expect(html).toContain(title)
    expect(html).toContain(message)
  })
})
