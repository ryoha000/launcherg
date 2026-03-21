<script lang='ts'>
  import type { ScanProgressState } from '@/components/Sidebar/useImportProgress.svelte'
  import { goto } from '@mateothegreat/svelte5-router'
  import { onDestroy } from 'svelte'
  import { get } from 'svelte/store'
  import ScanProgressDialog from '@/components/Sidebar/ScanProgressDialog.svelte'
  import Button from '@/components/UI/Button.svelte'
  import Input from '@/components/UI/Input.svelte'
  import InputPath from '@/components/UI/InputPath.svelte'
  import { useStorageSettingsMutation, useStorageSettingsQuery } from '@/lib/data/queries/storagePaths'

  let settings = $state({
    theme: 'dark',
    autoLaunch: false,
    showNotifications: true,
    defaultDirectory: '',
    maxRecentGames: '10',
  })

  let storageSettings = $state({
    imageStorageDir: '',
    downloadedGameStorageDir: '',
  })
  let storageSettingsInitialized = $state(false)
  let storageMessage = $state('')
  let storageError = $state('')
  let isOpenProgressPreview = $state(false)
  let progressPreviewStartedAt = $state<number | null>(null)
  let progressPreviewTimer: ReturnType<typeof setInterval> | null = null

  const previewPaths = [
    'D:\\Games\\DLsite\\2024\\VeryLongFolderName\\EpisodeOne\\installer\\setup.exe',
    'D:\\Games\\DLsite\\2024\\VeryLongFolderName\\EpisodeOne\\media\\cg\\scene_001.png',
    'D:\\Games\\DLsite\\2024\\VeryLongFolderName\\EpisodeTwo\\movie\\opening\\scene01\\thumbnail_preview.png',
    'D:\\Games\\DLsite\\2024\\VeryLongFolderName\\EpisodeTwo\\media\\voice\\voice_0003.ogg',
    'D:\\Games\\DLsite\\2024\\VeryLongFolderName\\EpisodeThree\\image\\banner\\banner_large_2x.png',
  ]

  const previewDurationMs = 14000

  const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value))
  const lerp = (from: number, to: number, ratio: number) => from + ((to - from) * ratio)

  const createPreviewState = (elapsedMs: number): ScanProgressState => {
    const clampedElapsed = clamp(elapsedMs, 0, previewDurationMs)
    const phase1End = 3400
    const phase2End = 7600
    const phase3End = 10800
    const phase4End = 12800

    const pickPath = (index: number) => previewPaths[index % previewPaths.length]

    if (clampedElapsed < phase1End) {
      const ratio = clampedElapsed / phase1End
      const discovered = Math.floor(lerp(0, 420, ratio))
      const judged = Math.floor(lerp(0, 210, ratio))
      const recognized = Math.floor(lerp(0, 52, ratio))
      return {
        startedAt: progressPreviewStartedAt,
        elapsedSeconds: Math.floor(clampedElapsed / 1000),
        explore: {
          status: 'running',
          discoveredCandidates: discovered,
          currentPath: pickPath(Math.floor(clampedElapsed / 220)),
          totalCandidates: null,
        },
        judge: {
          status: 'running',
          judgedCount: judged,
          recognizedCount: recognized,
          totalCandidates: null,
        },
        images: {
          status: 'idle',
          processedCount: 0,
          totalCount: null,
        },
      }
    }

    if (clampedElapsed < phase2End) {
      const ratio = (clampedElapsed - phase1End) / (phase2End - phase1End)
      const discovered = Math.floor(lerp(420, 1284, ratio))
      const judged = Math.floor(lerp(210, 913, ratio))
      const recognized = Math.floor(lerp(52, 241, ratio))
      return {
        startedAt: progressPreviewStartedAt,
        elapsedSeconds: Math.floor(clampedElapsed / 1000),
        explore: {
          status: 'running',
          discoveredCandidates: discovered,
          currentPath: pickPath(Math.floor(clampedElapsed / 180)),
          totalCandidates: null,
        },
        judge: {
          status: 'running',
          judgedCount: judged,
          recognizedCount: recognized,
          totalCandidates: null,
        },
        images: {
          status: 'idle',
          processedCount: 0,
          totalCount: null,
        },
      }
    }

    if (clampedElapsed < phase3End) {
      const ratio = (clampedElapsed - phase2End) / (phase3End - phase2End)
      const judged = Math.floor(lerp(913, 1284, ratio))
      const recognized = Math.floor(lerp(241, 298, ratio))
      return {
        startedAt: progressPreviewStartedAt,
        elapsedSeconds: Math.floor(clampedElapsed / 1000),
        explore: {
          status: 'done',
          discoveredCandidates: 1284,
          currentPath: pickPath(Math.floor(clampedElapsed / 220)),
          totalCandidates: 1284,
        },
        judge: {
          status: 'running',
          judgedCount: judged,
          recognizedCount: recognized,
          totalCandidates: 1284,
        },
        images: {
          status: 'idle',
          processedCount: 0,
          totalCount: null,
        },
      }
    }

    if (clampedElapsed < phase4End) {
      const ratio = (clampedElapsed - phase3End) / (phase4End - phase3End)
      const processed = Math.floor(lerp(0, 298, ratio))
      return {
        startedAt: progressPreviewStartedAt,
        elapsedSeconds: Math.floor(clampedElapsed / 1000),
        explore: {
          status: 'done',
          discoveredCandidates: 1284,
          currentPath: pickPath(Math.floor(clampedElapsed / 250)),
          totalCandidates: 1284,
        },
        judge: {
          status: 'done',
          judgedCount: 1284,
          recognizedCount: 298,
          totalCandidates: 1284,
        },
        images: {
          status: 'running',
          processedCount: processed,
          totalCount: 298,
        },
      }
    }

    return {
      startedAt: progressPreviewStartedAt,
      elapsedSeconds: Math.floor(clampedElapsed / 1000),
      explore: {
        status: 'done',
        discoveredCandidates: 1284,
        currentPath: pickPath(Math.floor(clampedElapsed / 260)),
        totalCandidates: 1284,
      },
      judge: {
        status: 'done',
        judgedCount: 1284,
        recognizedCount: 298,
        totalCandidates: 1284,
      },
      images: {
        status: 'done',
        processedCount: 298,
        totalCount: 298,
      },
    }
  }

  let progressPreview = $state<ScanProgressState>(createPreviewState(0))

  const stopProgressPreviewTimer = () => {
    if (progressPreviewTimer !== null) {
      clearInterval(progressPreviewTimer)
      progressPreviewTimer = null
    }
  }

  const advanceProgressPreview = () => {
    if (progressPreviewStartedAt === null) {
      return
    }
    progressPreview = createPreviewState(Date.now() - progressPreviewStartedAt)
  }

  const startProgressPreview = () => {
    stopProgressPreviewTimer()
    progressPreviewStartedAt = Date.now()
    progressPreview = createPreviewState(0)
    progressPreviewTimer = setInterval(advanceProgressPreview, 33)
  }

  const openProgressPreview = () => {
    isOpenProgressPreview = true
    startProgressPreview()
  }

  const closeProgressPreview = () => {
    isOpenProgressPreview = false
    stopProgressPreviewTimer()
    progressPreviewStartedAt = null
    progressPreview = createPreviewState(0)
  }

  const storageSettingsQuery = useStorageSettingsQuery()
  const storageSettingsMutation = useStorageSettingsMutation()

  $effect(() => {
    const data = $storageSettingsQuery.data
    if (!data || storageSettingsInitialized) {
      return
    }
    storageSettings = {
      imageStorageDir: data.imageStorageDir ?? '',
      downloadedGameStorageDir: data.downloadedGameStorageDir ?? '',
    }
    storageSettingsInitialized = true
  })

  function saveSettings() {
    storageError = ''
    storageMessage = ''
    void (async () => {
      try {
        const saved = await get(storageSettingsMutation).mutateAsync({
          imageStorageDir: storageSettings.imageStorageDir.trim() || null,
          downloadedGameStorageDir: storageSettings.downloadedGameStorageDir.trim() || null,
        })
        storageSettings = {
          imageStorageDir: saved.imageStorageDir ?? '',
          downloadedGameStorageDir: saved.downloadedGameStorageDir ?? '',
        }
        storageMessage = '保存しました'
      }
      catch (err) {
        storageError = err instanceof Error ? err.message : String(err)
      }
    })()
  }

  function resetSettings() {
    settings = {
      theme: 'dark',
      autoLaunch: false,
      showNotifications: true,
      defaultDirectory: '',
      maxRecentGames: '10',
    }
    storageError = ''
    storageMessage = ''
    const data = $storageSettingsQuery.data
    storageSettings = {
      imageStorageDir: data?.imageStorageDir ?? '',
      downloadedGameStorageDir: data?.downloadedGameStorageDir ?? '',
    }
  }

  function navigateToProcTailDebug() {
    goto('/debug/proctail')
  }

  function navigateToExtensionManager() {
    goto('/debug/extensionmanager')
  }

  function navigateToExtensionLog() {
    goto('/debug/extensionlog')
  }

  function navigateToImageQueue() {
    goto('/image-queue')
  }

  onDestroy(stopProgressPreviewTimer)
</script>

<div class='mx-auto h-full max-w-2xl overflow-y-auto p-6'>
  <div class='space-y-6'>
    <!-- テーマ設定 -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>表示設定</h2>
      <div class='space-y-3'>
        <div>
          <label for='theme-select' class='mb-2 block text-(sm text-secondary) font-medium'>テーマ</label>
          <select id='theme-select' bind:value={settings.theme} class='w-full border border-(border-primary) rounded bg-(bg-secondary) p-2 text-(text-primary)'>
            <option value='dark'>ダーク</option>
            <option value='light'>ライト</option>
            <option value='auto'>システム設定に従う</option>
          </select>
        </div>
      </div>
    </div>

    <!-- 起動設定 -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>起動設定</h2>
      <div class='space-y-3'>
        <label class='flex items-center'>
          <input type='checkbox' bind:checked={settings.autoLaunch} class='mr-2'>
          <span class='text-(text-secondary)'>システム起動時に自動起動</span>
        </label>
        <label class='flex items-center'>
          <input type='checkbox' bind:checked={settings.showNotifications} class='mr-2'>
          <span class='text-(text-secondary)'>通知を表示</span>
        </label>
      </div>
    </div>

    <!-- ゲーム設定 -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>ゲーム設定</h2>
      <div class='space-y-3'>
        <div>
          <Input bind:value={settings.defaultDirectory} placeholder='ゲームファイルのデフォルトディレクトリ' />
        </div>
        <div>
          <Input bind:value={settings.maxRecentGames} placeholder='10' />
        </div>
      </div>
    </div>

    <!-- 保存先設定 -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>保存先設定</h2>
      <div class='space-y-4'>
        <div>
          <InputPath
            bind:path={storageSettings.imageStorageDir}
            label='画像保存先'
            placeholder='既定の保存先を使用'
            directory={true}
            withFilter={false}
          />
          <p class='mt-2 text-(sm text-secondary)'>
            thumbnails と icons のみが対象です。既存データは移動されません。
          </p>
        </div>
        <div>
          <InputPath
            bind:path={storageSettings.downloadedGameStorageDir}
            label='ダウンロードゲーム保存先'
            placeholder='既定の保存先を使用'
            directory={true}
            withFilter={false}
          />
          <p class='mt-2 text-(sm text-secondary)'>
            downloaded_games のみが対象です。既存データは移動されません。
          </p>
        </div>
        <div class='border border-(border-primary) rounded bg-(bg-secondary) p-3 text-(sm text-secondary)'>
          保存先が無効な場合は保存に失敗します。固定パスへの自動フォールバックは行いません。
        </div>
        {#if storageMessage}
          <p class='text-text-success text-(sm)'>{storageMessage}</p>
        {/if}
        {#if storageError}
          <p class='text-text-error text-(sm)'>{storageError}</p>
        {/if}
      </div>
    </div>

    <!-- 拡張機能のログ -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>拡張機能のログ</h2>
      <div class='space-y-3'>
        <Button variant='normal' onclick={navigateToExtensionLog} text='拡張機能のログを表示' />
      </div>
    </div>

    <!-- 拡張機能 -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>拡張機能</h2>
      <div class='space-y-3'>
        <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
          <h3 class='mb-2 text-(base text-primary) font-medium'>ブラウザ拡張機能</h3>
          <p class='mb-3 text-(sm text-secondary)'>DMM GamesやDLsiteからゲーム情報を自動取得します</p>
          <Button onclick={navigateToExtensionManager} text='拡張機能を管理' />
        </div>
      </div>
    </div>

    <!-- デバッグ設定 -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>デバッグ</h2>
      <div class='space-y-3'>
        <Button
          variant='normal'
          onclick={openProgressPreview}
          text='進捗ダイアログをプレビュー'
        />
        <Button variant='normal' onclick={navigateToProcTailDebug} text='ProcTailデバッグ画面' />
        <Button variant='normal' onclick={navigateToImageQueue} text='画像保存キュー' />
      </div>
    </div>

    <!-- ボタン -->
    <div class='flex gap-3 pt-4'>
      <Button onclick={saveSettings} text='設定を保存' />
      <Button variant='normal' onclick={resetSettings} text='リセット' />
    </div>
  </div>
</div>

<ScanProgressDialog
  isOpen={isOpenProgressPreview}
  progress={progressPreview}
  panelClass='max-w-120'
  onclose={closeProgressPreview}
/>
