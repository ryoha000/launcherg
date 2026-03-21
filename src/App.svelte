<script lang='ts'>
  import { goto, Router } from '@mateothegreat/svelte5-router'
  import { QueryClientProvider } from '@tanstack/svelte-query'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { getCurrent, onOpenUrl } from '@tauri-apps/plugin-deep-link'
  import { onMount } from 'svelte'
  import ImportDropFiles from '@/components/Home/ImportDropFiles.svelte'
  import Titlebar from '@/components/UI/Titlebar/Titlebar.svelte'
  import Layout from '@/layouts/Layout.svelte'
  import {
    commandBackfillThumbnailSizes,
    commandGetWorkDetailsByWorkId,
    commandListWorkLnks,
    commandProcessPendingExeLinks,
    commandShowOsNotification,
  } from '@/lib/command'
  import { queryClient } from '@/lib/data/queryClient'
  import { queryKeys } from '@/lib/data/queryKeys'
  import { buildWorkOpenDeepLink, parseDeepLinkUrl } from '@/lib/deepLink'
  import { useEvent } from '@/lib/event'
  import { runLocalMigrationV3 } from '@/lib/migrations/workCenteredV3'
  import { registerErogamescapeInformations } from '@/lib/registerErogamescapeInformations'
  import { initializeAllGameCache } from '@/lib/scrape/scrapeAllGame'
  import { showErrorToast, showInfoToast } from '@/lib/toast'
  import { routes } from '@/router/route'
  import { sidebarWorks } from '@/store/sidebarWorks'
  import { initialize, routeLoaded } from '@/store/tabs'

  const appEvent = useEvent()
  const setDetailPromise = $derived(registerErogamescapeInformations())
  const notifiedDownloads = new Set<string>()
  const activeDownloadNotifications = new Set<string>()
  let pendingNotificationTarget = $state<{ workId: string, issuedAt: number, activationUrl: string } | null>(null)
  let focusUnlisten: (() => void) | null = null
  let deepLinkUnlisten: (() => void) | null = null

  const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

  const focusMainWindow = async () => {
    const appWindow = getCurrentWindow()
    try {
      await appWindow.unminimize()
    }
    catch (error) {
      console.warn('failed to unminimize window', error)
    }

    try {
      await appWindow.setFocus()
    }
    catch (error) {
      console.warn('failed to focus window', error)
    }
  }

  const handleDeepLinkUrl = async (rawUrl: string): Promise<boolean> => {
    const target = parseDeepLinkUrl(rawUrl)
    if (!target) {
      console.error('invalid deep link url', { rawUrl })
      return false
    }

    pendingNotificationTarget = null

    await goto(target.path)
    await focusMainWindow()
    return true
  }

  const handleDeepLinkUrls = async (urls: string[]) => {
    for (const rawUrl of urls) {
      if (await handleDeepLinkUrl(rawUrl))
        return
    }
  }

  const notifyWhenPlayable = async (workId: string) => {
    if (notifiedDownloads.has(workId) || activeDownloadNotifications.has(workId)) {
      return
    }

    activeDownloadNotifications.add(workId)
    try {
      const workDetail = await commandGetWorkDetailsByWorkId(workId)
      const notificationTitle = workDetail?.title?.trim() || workId

      let isPlayable = false
      for (let i = 0; i < 20; i += 1) {
        const lnks = await commandListWorkLnks(workId)
        if (lnks.length > 0) {
          isPlayable = true
          break
        }
        await sleep(500)
      }

      if (!isPlayable) {
        return
      }

      notifiedDownloads.add(workId)
      const activationUrl = buildWorkOpenDeepLink(workId, {
        play: true,
        gamename: notificationTitle,
      })
      pendingNotificationTarget = { workId, issuedAt: Date.now(), activationUrl }
      await commandShowOsNotification(
        'Launcherg',
        `「${notificationTitle}」をプレイできるようになりました`,
        activationUrl,
      )
    }
    catch (error) {
      console.error('failed to show download notification', { workId, error })
      pendingNotificationTarget = null
    }
    finally {
      activeDownloadNotifications.delete(workId)
    }
  }

  const activatePendingNotification = async () => {
    const pending = pendingNotificationTarget
    if (!pending) {
      return
    }

    if (Date.now() - pending.issuedAt > 1000 * 60 * 2) {
      pendingNotificationTarget = null
      return
    }

    pendingNotificationTarget = null
    await handleDeepLinkUrl(pending.activationUrl)
  }

  const init = async () => {
    await runLocalMigrationV3()
    await commandProcessPendingExeLinks()
    initialize()
    initializeAllGameCache()
    commandBackfillThumbnailSizes()
  }

  onMount(() => {
    init()
    const appWindow = getCurrentWindow()
    void appWindow.onFocusChanged(async (focused) => {
      if (!focused) {
        return
      }

      await activatePendingNotification()
    }).then((unlisten) => {
      focusUnlisten = unlisten
    })

    void onOpenUrl((urls) => {
      void handleDeepLinkUrls(urls)
    }).then((unlisten) => {
      deepLinkUnlisten = unlisten
    })

    void getCurrent().then((urls) => {
      if (!urls) {
        return
      }

      void handleDeepLinkUrls(urls)
    })

    void appEvent.startListen('appSignal:showMessage', ({ event }) => {
      if (event.type !== 'showMessage')
        return

      showInfoToast(event.payload.message)
    })

    void appEvent.startListen('appSignal:showErrorMessage', ({ event }) => {
      if (event.type !== 'showErrorMessage')
        return

      showErrorToast(event.payload.message)
    })

    void appEvent.startListen('appSignal:refetchWorks', ({ event }) => {
      if (event.type !== 'refetchWorks')
        return

      void sidebarWorks.refetch()
    })

    void appEvent.startListen('appSignal:refetchWork', ({ event }) => {
      if (event.type !== 'refetchWork')
        return

      void queryClient.invalidateQueries({ queryKey: queryKeys.workDetails.byId(event.payload.workId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.workLnk.byId(event.payload.workId) })
      void notifyWhenPlayable(event.payload.workId)
    })

    return () => {
      focusUnlisten?.()
      deepLinkUnlisten?.()
      appEvent.stopAll()
    }
  })

</script>

<main class='grid grid-rows-[auto_1fr] h-full w-full bg-(bg-primary) font-sans'>
  <Titlebar />
  {#await setDetailPromise then _}
    <QueryClientProvider client={queryClient}>
      <Layout>
        <Router {routes} hooks={{ post: routeLoaded }} />
      </Layout>
    </QueryClientProvider>
  {/await}
  <ImportDropFiles />
</main>
