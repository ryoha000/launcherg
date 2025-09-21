<script lang='ts'>
  import type { RegistryKeyInfo } from '@/lib/command'
  import type { EventPayloadMap } from '@/lib/event/types'
  import { listen } from '@tauri-apps/api/event'
  import { onMount } from 'svelte'
  import Button from '@/components/UI/Button.svelte'
  import Input from '@/components/UI/Input.svelte'
  import {
    commandCheckRegistryKeys,
    commandCopyExtensionForDevelopment,
    commandGenerateExtensionPackage,
    commandGetDevExtensionInfo,
    commandGetExtensionPackageInfo,
    commandGetSyncStatus,
    commandRemoveRegistryKeys,
    commandSetupNativeMessagingHost,
  } from '@/lib/command'
  import { showErrorToast, showInfoToast } from '@/lib/toast'

  // 拡張機能の状態
  let extensionStatus = $state<'connected' | 'disconnected' | 'unknown'>('unknown')
  let syncStatus = $state<any>(null)
  let loading = $state(false)
  let detailedConnectionStatus = $state<string | null>(null)
  let errorMessage = $state<string | null>(null)

  // 同期履歴
  let syncHistory = $state<any[]>([])

  // 手動同期テスト用
  let testSyncData = $state({
    store: 'DMM' as 'DMM' | 'DLSite',
    sampleGames: [
      {
        store_id: 'test_001',
        title: 'テストゲーム1',
        purchase_url: 'https://dlsoft.dmm.co.jp/detail/test_001',
        additional_data: {
          erogamescape_id: '12345',
        },
      },
    ],
  })

  // 拡張機能パッケージ関連の状態
  let packageInfo = $state<any>(null)
  let packageGenerating = $state(false)
  let setupRunning = $state(false)
  let setupOutput = $state<string>('')
  let setupStep = $state<'idle' | 'package' | 'guide' | 'host' | 'complete'>('idle')
  let installGuideVisible = $state(false)
  let devExtensionPath = $state<string | null>(null)
  let devCopyInProgress = $state(false)

  // Extension ID管理
  let customExtensionId = $state<string>('')
  let savedExtensionId = $state<string | null>(null)

  // レジストリキー管理
  let registryKeys = $state<RegistryKeyInfo[]>([])
  let registryKeysLoading = $state(false)

  const getDetailedStatusMessage = (connectionStatus: string) => {
    switch (connectionStatus) {
      case 'connected': return '正常に接続されています'
      case 'connecting': return '接続中です'
      case 'host_not_found': return 'Native Messaging Hostの実行ファイルが見つかりません'
      case 'host_startup_failed': return 'Native Messaging Hostプロセスの起動に失敗しました'
      case 'health_check_timeout': return 'ヘルスチェックがタイムアウトしました'
      case 'health_check_failed': return 'ヘルスチェックでエラーが発生しました'
      case 'communication_error': return '通信エラーが発生しました'
      case 'process_termination_error': return 'プロセス終了時にエラーが発生しました'
      case 'unknown_error': return '不明なエラーが発生しました'
      default: return '状態不明'
    }
  }

  // レジストリキー情報を読み込み
  const loadRegistryKeys = async () => {
    registryKeysLoading = true
    try {
      registryKeys = await commandCheckRegistryKeys()
    }
    catch (e) {
      console.error('Failed to load registry keys:', e)
      showErrorToast(`レジストリキーの読み込みに失敗: ${e}`)
    }
    finally {
      registryKeysLoading = false
    }
  }

  const loadExtensionStatus = async () => {
    loading = true

    try {
      // 接続チェックを実行（PubSubで状態変化が通知される）
      syncStatus = await commandGetSyncStatus()

      // レスポンスから最終的な詳細情報を設定
      detailedConnectionStatus = syncStatus.connection_status
      errorMessage = syncStatus.error_message
      extensionStatus = syncStatus.is_running ? 'connected' : 'disconnected'

      // 最終結果のトーストメッセージ（PubSubの中間通知とは別）
      if (syncStatus.is_running) {
        showInfoToast('拡張機能との接続が完了しました')
      }
      else {
        const statusMessage = getDetailedStatusMessage(syncStatus.connection_status)
        showErrorToast(`接続チェック完了: ${statusMessage}`)
      }
    }
    catch (e) {
      console.error('Failed to get sync status:', e)
      extensionStatus = 'disconnected'
      detailedConnectionStatus = 'unknown_error'
      errorMessage = String(e)
      showErrorToast(`拡張機能の状態取得に失敗: ${e}`)
    }
    finally {
      loading = false
    }
  }

  const testManualSync = async () => {
    // TODO: sync_dl_store_games_batch コマンドが削除されたため、この機能は現在利用できません
    showErrorToast('手動同期機能は現在利用できません')
  }

  // 拡張機能パッケージ生成
  const generatePackage = async () => {
    packageGenerating = true
    try {
      packageInfo = await commandGenerateExtensionPackage()
      showInfoToast('拡張機能パッケージを生成しました')
    }
    catch (e) {
      showErrorToast(`パッケージ生成に失敗: ${e}`)
    }
    finally {
      packageGenerating = false
    }
  }

  // パッケージ情報読み込み
  const loadPackageInfo = async () => {
    try {
      packageInfo = await commandGetExtensionPackageInfo()
    }
    catch (e) {
      console.error('Failed to load package info:', e)
    }
  }

  // 開発用拡張機能情報読み込み
  const loadDevExtensionInfo = async () => {
    try {
      devExtensionPath = await commandGetDevExtensionInfo()
    }
    catch (e) {
      console.error('Failed to load dev extension info:', e)
    }
  }

  // 開発用拡張機能コピー
  const copyExtensionForDev = async () => {
    devCopyInProgress = true
    try {
      devExtensionPath = await commandCopyExtensionForDevelopment()
      showInfoToast('開発用拡張機能フォルダを作成しました')
    }
    catch (e) {
      showErrorToast(`開発用コピーに失敗: ${e}`)
    }
    finally {
      devCopyInProgress = false
    }
  }

  // Native Messaging Host セットアップ
  const setupNativeHost = async () => {
    setupRunning = true
    setupOutput = ''
    try {
      // 保存されたExtension IDがある場合はそれを使用
      let setupOptions = {}
      if (savedExtensionId) {
        setupOptions = { extensionId: savedExtensionId }
      }

      const output = await commandSetupNativeMessagingHost(setupOptions)
      setupOutput = output
      showInfoToast('Native Messaging Host のセットアップが完了しました')
      // セットアップ後に接続状況を更新
      await loadExtensionStatus()
    }
    catch (e) {
      setupOutput = `エラー: ${e}`
      showErrorToast(`セットアップに失敗: ${e}`)
    }
    finally {
      setupRunning = false
    }
  }

  // ワンクリックセットアップ（Native Messaging Host設定のみ）
  const oneClickSetup = async () => {
    setupStep = 'host'

    try {
      // Native Messaging Host セットアップ
      await setupNativeHost()

      // 完了
      setupStep = 'complete'
      showInfoToast('Native Messaging Host のセットアップが完了しました！')

      // レジストリキー情報を更新
      await loadRegistryKeys()
    }
    catch (e) {
      showErrorToast(`セットアップに失敗: ${e}`)
      setupStep = 'idle'
    }
  }

  // レジストリキーを削除
  const removeRegistryKeys = async () => {
    try {
      const results = await commandRemoveRegistryKeys()
      results.forEach((result) => {
        if (result.includes('successfully') || result.includes('removed')) {
          showInfoToast(result)
        }
        else {
          showErrorToast(result)
        }
      })

      // レジストリキー情報を更新
      await loadRegistryKeys()
    }
    catch (e) {
      showErrorToast(`レジストリキーの削除に失敗: ${e}`)
    }
  }

  onMount(() => {
    let unlisten: (() => void) | null = null

    // 保存されたExtension IDを読み込み
    const storedId = localStorage.getItem('launcherg_extension_id')
    if (storedId) {
      savedExtensionId = storedId
      customExtensionId = storedId
    }

    const setupListener = async () => {
      // PubSubイベントリスナーを設定
      unlisten = await listen<EventPayloadMap['extension-connection-status']>('extension-connection-status', ({ payload }) => {
        // console.log('Extension connection status update:', payload)

        // 状態を更新
        detailedConnectionStatus = payload.connectionStatus
        errorMessage = payload.errorMessage ?? null
        extensionStatus = payload.isRunning ? 'connected' : 'disconnected'

        // UI更新のために時刻を記録
        if (syncStatus) {
          syncStatus.connection_status = payload.connectionStatus
          syncStatus.error_message = payload.errorMessage ?? null
          syncStatus.is_running = payload.isRunning
        }
      })

      // 初回データ読み込み
      await loadExtensionStatus()
      await loadPackageInfo()
      await loadDevExtensionInfo()
      await loadRegistryKeys()
    }

    setupListener()

    // クリーンアップ関数を返す
    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  })

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString('ja-JP')
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'connected': return 'text-green-600'
      case 'disconnected': return 'text-red-600'
      default: return 'text-gray-600'
    }
  }

  const getStatusText = (status: string) => {
    switch (status) {
      case 'connected': return '接続済み'
      case 'disconnected': return '切断'
      default: return '確認中...'
    }
  }

  const getDetailedStatusColor = (connectionStatus: string) => {
    switch (connectionStatus) {
      case 'connected': return 'text-green-600'
      case 'connecting': return 'text-yellow-600'
      case 'host_not_found':
      case 'host_startup_failed':
      case 'health_check_timeout':
      case 'health_check_failed':
      case 'communication_error':
      case 'process_termination_error':
      case 'unknown_error':
        return 'text-red-600'
      default: return 'text-gray-600'
    }
  }
</script>

<div class='mx-auto h-full max-w-4xl overflow-y-auto p-6'>
  <div class='space-y-6'>
    <!-- Header -->
    <div class='flex items-center justify-between'>
      <h1 class='text-(2xl text-primary) font-bold'>ブラウザ拡張機能管理</h1>
      <div class='flex gap-2'>
        <Button variant='normal' onclick={loadExtensionStatus} text='再接続' />
        <div class='flex items-center gap-2'>
          <div class='h-3 w-3 rounded-full' class:bg-green-500={extensionStatus === 'connected'} class:bg-red-500={extensionStatus === 'disconnected'} class:bg-gray-500={extensionStatus === 'unknown'}></div>
          <span class='text-(sm text-secondary)'>{getStatusText(extensionStatus)}</span>
        </div>
      </div>
    </div>

    <!-- 接続状況 -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <h3 class='mb-3 text-(lg text-primary) font-semibold'>接続状況</h3>
      <div class='flex items-center justify-between'>
        <div class='space-y-2'>
          <div class='flex items-center gap-2'>
            <span class='text-(sm text-secondary)'>拡張機能:</span>
            <span class={`font-medium ${getStatusColor(extensionStatus)}`}>
              {getStatusText(extensionStatus)}
            </span>
          </div>

          {#if detailedConnectionStatus}
            <div class='flex items-center gap-2'>
              <span class='text-(sm text-secondary)'>詳細状態:</span>
              <span class={`text-sm font-medium ${getDetailedStatusColor(detailedConnectionStatus)}`}>
                {getDetailedStatusMessage(detailedConnectionStatus)}
              </span>
            </div>
          {/if}

          {#if errorMessage}
            <div class='mt-2 border border-red-200 rounded bg-red-50 p-2 dark:border-red-800 dark:bg-red-900/20'>
              <div class='flex items-start gap-2'>
                <span class='text-sm text-red-600 font-medium'>エラー詳細:</span>
                <span class='break-all text-sm text-red-700 dark:text-red-300'>
                  {errorMessage}
                </span>
              </div>
            </div>
          {/if}

          {#if syncStatus}
            <div class='flex items-center gap-2'>
              <span class='text-(sm text-secondary)'>最終同期:</span>
              <span class='text-(sm text-primary)'>
                {syncStatus.last_sync ? formatDate(syncStatus.last_sync) : '未同期'}
              </span>
            </div>
            <div class='flex items-center gap-2'>
              <span class='text-(sm text-secondary)'>総同期数:</span>
              <span class='text-(sm text-primary) font-medium'>
                {syncStatus.total_synced}
              </span>
            </div>
          {/if}
        </div>
        <Button
          text='更新'
          variant='normal'
          onclick={loadExtensionStatus}
          disabled={loading}
        />
      </div>
    </div>

    <!-- テスト機能 -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <h3 class='mb-3 text-(lg text-primary) font-semibold'>テスト機能</h3>
      <div class='space-y-4'>
        <div class='space-y-2'>
          <label for='test-store' class='text-(base text-primary)'>テストストア</label>
          <select
            id='test-store'
            bind:value={testSyncData.store}
            class='w-full border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)'
          >
            <option value='DMM'>DMM Games</option>
            <option value='DLSite'>DLsite</option>
          </select>
        </div>
        <Button
          text='テスト同期実行'
          onclick={testManualSync}
          disabled={loading}
        />
        <p class='text-(sm text-secondary)'>
          ※ テスト用のサンプルデータを使用して同期機能をテストします
        </p>
      </div>
    </div>

    <!-- 同期履歴 -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <h3 class='mb-3 text-(lg text-primary) font-semibold'>同期履歴</h3>
      {#if syncHistory.length === 0}
        <div class='py-8 text-center'>
          <p class='text-(lg text-secondary)'>同期履歴はありません</p>
          <p class='mt-1 text-(sm text-tertiary)'>テスト同期を実行すると履歴が表示されます</p>
        </div>
      {:else}
        <div class='max-h-96 overflow-y-auto space-y-2'>
          <div class='mb-2 px-2 text-(sm text-secondary)'>
            {syncHistory.length}件の履歴が見つかりました
          </div>
          {#each syncHistory as history, index}
            <div class='border-border-secondary border rounded bg-(bg-tertiary) p-3 transition-colors hover:bg-(bg-secondary)'>
              <div class='mb-2 flex items-center justify-between'>
                <div class='flex items-center gap-2'>
                  <span class='text-(xs text-tertiary) font-mono'>#{syncHistory.length - index}</span>
                  <span class='rounded bg-(bg-primary) px-2 py-1 text-(xs bg-secondary text-primary) font-medium'>
                    {history.store} - {history.type}
                  </span>
                </div>
                <span class='text-(xs text-secondary) font-mono'>
                  {formatDate(history.timestamp)}
                </span>
              </div>
              <div class='flex items-center gap-4 text-(sm text-secondary)'>
                <span class='text-green-600'>成功: {history.success_count}</span>
                <span class='text-red-600'>エラー: {history.error_count}</span>
              </div>
              {#if history.errors && history.errors.length > 0}
                <div class='mt-2 text-(xs text-secondary)'>
                  <details>
                    <summary class='cursor-pointer'>エラー詳細</summary>
                    <div class='mt-1 pl-4'>
                      {#each history.errors as error}
                        <div class='text-red-600'>{error}</div>
                      {/each}
                    </div>
                  </details>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- 拡張機能パッケージ -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <h3 class='mb-3 text-(lg text-primary) font-semibold'>拡張機能パッケージ</h3>
      <div class='space-y-4'>
        {#if packageInfo}
          <div class='border-border-secondary border rounded bg-(bg-tertiary) p-3'>
            <div class='mb-2 flex items-center justify-between'>
              <h4 class='text-(text-primary) font-medium'>{packageInfo.manifest_info.name}</h4>
              <span class='rounded bg-(bg-primary) px-2 py-1 text-(xs text-secondary)'>
                v{packageInfo.manifest_info.version}
              </span>
            </div>
            <p class='mb-2 text-(sm text-secondary)'>
              {packageInfo.manifest_info.description}
            </p>
            <div class='grid grid-cols-2 gap-4 text-sm'>
              <div>
                <span class='text-(text-secondary)'>Extension ID:</span>
                <div class='mt-1 rounded bg-(bg-primary) p-1 text-(xs text-primary) font-mono'>
                  {packageInfo.manifest_info.extension_id}
                </div>
              </div>
              <div>
                <span class='text-(text-secondary)'>パッケージサイズ:</span>
                <div class='mt-1 text-(text-primary)'>
                  {Math.round((new Blob([packageInfo.package_path]).size) / 1024)}KB
                </div>
              </div>
            </div>
            <div class='mt-3 flex gap-2'>
              <Button
                variant='normal'
                text='パッケージをダウンロード'
                onclick={() => {
                  // ダウンロード機能（実装予定）
                  showInfoToast('ダウンロード機能は実装予定です')
                }}
              />
              <Button
                variant='normal'
                text='再生成'
                onclick={generatePackage}
                disabled={packageGenerating}
              />
            </div>
          </div>
        {:else}
          <div class='py-8 text-center'>
            <p class='mb-2 text-(lg text-secondary)'>パッケージが生成されていません</p>
            <p class='mb-4 text-(sm text-tertiary)'>
              拡張機能をパッケージ化して配布用のZIPファイルを作成します
            </p>
            <Button
              text={packageGenerating ? '生成中...' : 'パッケージを生成'}
              onclick={generatePackage}
              disabled={packageGenerating}
            />
          </div>
        {/if}
      </div>
    </div>

    <!-- 開発環境用 -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <h3 class='mb-3 text-(lg text-primary) font-semibold'>開発環境用</h3>
      <div class='space-y-4'>
        {#if devExtensionPath}
          <div class='border-border-secondary border rounded bg-(bg-tertiary) p-3'>
            <div class='mb-2 flex items-center justify-between'>
              <h4 class='text-(text-primary) font-medium'>開発用拡張機能フォルダ</h4>
              <span class='rounded bg-green-100 px-2 py-1 text-xs text-green-600'>準備完了</span>
            </div>
            <p class='mb-2 text-(sm text-secondary)'>
              Chrome/Edgeの「パッケージ化されていない拡張機能を読み込む」で以下のフォルダを選択してください
            </p>
            <div class='border rounded bg-(bg-primary) p-2'>
              <code class='break-all text-(xs text-primary)'>{devExtensionPath}</code>
            </div>
            <div class='mt-3 flex gap-2'>
              <Button
                variant='normal'
                text='フォルダを開く'
                onclick={() => {
                  // フォルダを開く機能（実装予定）
                  showInfoToast('フォルダを開く機能は実装予定です')
                }}
              />
              <Button
                variant='normal'
                text='再作成'
                onclick={copyExtensionForDev}
                disabled={devCopyInProgress}
              />
            </div>
          </div>
        {:else}
          <div class='py-8 text-center'>
            <p class='mb-2 text-(lg text-secondary)'>開発用フォルダが作成されていません</p>
            <p class='mb-4 text-(sm text-tertiary)'>
              拡張機能をビルドして、ブラウザですぐに読み込めるフォルダを作成します
            </p>
            <Button
              text={devCopyInProgress ? 'コピー中...' : '開発用フォルダを作成'}
              onclick={copyExtensionForDev}
              disabled={devCopyInProgress}
            />
          </div>
        {/if}

        <!-- 開発のヒント -->
        <div class='border border-blue-200 rounded bg-blue-50 p-3 dark:border-blue-800 dark:bg-blue-900/20'>
          <h4 class='mb-2 text-sm text-blue-800 font-medium dark:text-blue-200'>💡 開発のヒント</h4>
          <ul class='text-xs text-blue-700 space-y-1 dark:text-blue-300'>
            <li>• この機能は開発環境でのテスト用です</li>
            <li>• フォルダ内容を直接編集してリロードで確認できます</li>
            <li>• 本格的な配布にはZIPパッケージ機能をご利用ください</li>
          </ul>
        </div>
      </div>
    </div>

    <!-- インストールガイド -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <div class='mb-3 flex items-center justify-between'>
        <h3 class='text-(lg text-primary) font-semibold'>インストールガイド</h3>
        <Button
          variant='normal'
          text={installGuideVisible ? '手順を隠す' : '手順を表示'}
          onclick={() => installGuideVisible = !installGuideVisible}
        />
      </div>

      {#if installGuideVisible}
        <div class='space-y-4'>
          <!-- ステップ1: 拡張機能のインストール -->
          <div class='border-border-secondary border rounded p-3'>
            <div class='mb-2 flex items-center gap-2'>
              <div class='h-6 w-6 flex items-center justify-center rounded-full bg-blue-500 text-sm text-white font-bold'>1</div>
              <h4 class='text-(text-primary) font-medium'>ブラウザ拡張機能のインストール</h4>
            </div>
            <div class='ml-8 text-(sm text-secondary) space-y-2'>
              <p>1. 上記の「パッケージを生成」ボタンでZIPファイルを作成</p>
              <p>2. Chrome/Edgeで <code class='rounded bg-(bg-primary) px-1'>chrome://extensions/</code> を開く</p>
              <p>3. 「デベロッパーモード」を有効化</p>
              <p>4. 「パッケージ化されていない拡張機能を読み込む」をクリック</p>
              <p>5. 解凍したdistフォルダを選択</p>
            </div>
          </div>

          <!-- ステップ2: Native Messaging Host のセットアップ -->
          <div class='border-border-secondary border rounded p-3'>
            <div class='mb-2 flex items-center gap-2'>
              <div class='h-6 w-6 flex items-center justify-center rounded-full bg-green-500 text-sm text-white font-bold'>2</div>
              <h4 class='text-(text-primary) font-medium'>Native Messaging Host セットアップ</h4>
            </div>
            <div class='ml-8 text-(sm text-secondary) space-y-2'>
              <p>1. 下記の「セットアップを実行」ボタンをクリック</p>
              <p>2. PowerShellが管理者権限で実行されます</p>
              <p>3. 拡張機能IDが自動検出され、レジストリに登録されます</p>
              <div class='mt-2 border border-blue-200 rounded bg-blue-50 p-2 dark:border-blue-800 dark:bg-blue-900/20'>
                <p class='text-xs text-blue-800 dark:text-blue-200'>
                  ℹ️ 管理者権限が必要です。UAC確認ダイアログが表示されたら「はい」をクリックしてください。
                </p>
              </div>
            </div>
          </div>

          <!-- ステップ3: 動作確認 -->
          <div class='border-border-secondary border rounded p-3'>
            <div class='mb-2 flex items-center gap-2'>
              <div class='h-6 w-6 flex items-center justify-center rounded-full bg-purple-500 text-sm text-white font-bold'>3</div>
              <h4 class='text-(text-primary) font-medium'>動作確認</h4>
            </div>
            <div class='ml-8 text-(sm text-secondary) space-y-2'>
              <p>1. DMM Games または DLsite のライブラリページを開く</p>
              <p>2. 拡張機能アイコンをクリック</p>
              <p>3. 「手動同期」をクリックしてテスト</p>
              <p>4. 上記の「再接続」ボタンで接続状況を確認</p>
            </div>
          </div>

          <!-- 対応サイト情報 -->
          <div class='border-border-secondary border rounded bg-(bg-tertiary) p-3'>
            <h4 class='mb-2 text-(text-primary) font-medium'>対応サイト</h4>
            <ul class='text-(sm text-secondary) space-y-1'>
              <li class='flex items-center gap-2'>
                <div class='h-2 w-2 rounded-full bg-green-500'></div>
                DMM Games (dlsoft.dmm.co.jp) - 購入済みゲーム一覧
              </li>
              <li class='flex items-center gap-2'>
                <div class='h-2 w-2 rounded-full bg-green-500'></div>
                DLsite (www.dlsite.com) - マイライブラリ
              </li>
            </ul>
          </div>
        </div>
      {:else}
        <div class='py-4 text-center'>
          <p class='text-(sm text-secondary)'>
            ブラウザ拡張機能をインストールしてDMM/DLsiteから自動同期
          </p>
          <p class='mt-1 text-(xs text-tertiary)'>
            「手順を表示」をクリックして詳細なセットアップ手順をご確認ください
          </p>
        </div>
      {/if}
    </div>

    <!-- Extension ID設定 -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <h3 class='mb-3 text-(lg text-primary) font-semibold'>Extension ID 設定</h3>
      <div class='space-y-4'>
        <p class='text-(sm text-secondary)'>
          ブラウザにインストールした拡張機能のIDを入力してください。
          拡張機能管理ページ（chrome://extensions/）で確認できます。
        </p>

        <div class='space-y-2'>
          <div class='mb-2 text-(base text-primary)'>Extension ID</div>
          <div class='flex gap-2'>
            <div class='flex-1'>
              <Input
                bind:value={customExtensionId}
                placeholder='例: abcdefghijklmnopqrstuvwx'
              />
            </div>
            <Button
              text='保存'
              onclick={async () => {
                if (customExtensionId.trim()) {
                  savedExtensionId = customExtensionId.trim()
                  localStorage.setItem('launcherg_extension_id', savedExtensionId)
                  showInfoToast('Extension IDを保存しました')
                }
                else {
                  showErrorToast('Extension IDを入力してください')
                }
              }}
              disabled={!customExtensionId.trim()}
            />
          </div>
        </div>

        {#if savedExtensionId}
          <div class='border-border-secondary border rounded bg-(bg-tertiary) p-3'>
            <div class='flex items-center justify-between'>
              <div>
                <p class='text-(sm text-secondary)'>保存されたExtension ID:</p>
                <p class='mt-1 text-(sm text-primary) font-mono'>{savedExtensionId}</p>
              </div>
              <Button
                variant='normal'
                text='削除'
                onclick={() => {
                  savedExtensionId = null
                  customExtensionId = ''
                  localStorage.removeItem('launcherg_extension_id')
                  showInfoToast('Extension IDを削除しました')
                }}
              />
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- 自動セットアップ -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <h3 class='mb-3 text-(lg text-primary) font-semibold'>Native Messaging Host セットアップ</h3>

      <!-- セットアップ内容の説明 -->
      <div class='border-border-secondary mb-4 border rounded bg-(bg-tertiary) p-3'>
        <h4 class='mb-2 text-(sm text-primary) font-medium'>🔧 セットアップで実行される処理</h4>
        <div class='text-(xs text-secondary) space-y-1'>
          <div class='flex items-start gap-2'>
            <span class='text-blue-600'>1.</span>
            <span>Native Messaging Host 実行ファイル（native-messaging-host.exe）の存在確認・自動ビルド</span>
          </div>
          <div class='flex items-start gap-2'>
            <span class='text-blue-600'>2.</span>
            <span>マニフェストファイル（native-messaging-manifest.json）を読み込み</span>
          </div>
          <div class='flex items-start gap-2'>
            <span class='text-blue-600'>3.</span>
            <span>Extension ID を設定（カスタム設定またはブラウザから自動検出）</span>
          </div>
          <div class='flex items-start gap-2'>
            <span class='text-blue-600'>4.</span>
            <span>実行ファイルの絶対パスを設定</span>
          </div>
          <div class='flex items-start gap-2'>
            <span class='text-blue-600'>5.</span>
            <span>更新されたマニフェストを native-messaging-manifest-installed.json として保存</span>
          </div>
          <div class='flex items-start gap-2'>
            <span class='text-blue-600'>6.</span>
            <span class='text-green-600 font-medium'>Chrome レジストリキーを作成・登録</span>
          </div>
          <div class='flex items-start gap-2'>
            <span class='text-blue-600'>7.</span>
            <span class='text-green-600 font-medium'>Edge レジストリキーを作成・登録</span>
          </div>
        </div>
        <div class='mt-2 border border-yellow-200 rounded bg-yellow-50 p-2 dark:border-yellow-800 dark:bg-yellow-900/20'>
          <p class='text-xs text-yellow-800 dark:text-yellow-200'>
            <span class='font-medium'>レジストリキー:</span><br />
            • HKCU\Software\Google\Chrome\NativeMessagingHosts\moe.ryoha.launcherg.extension_host<br />
            • HKCU\Software\Microsoft\Edge\NativeMessagingHosts\moe.ryoha.launcherg.extension_host
          </p>
        </div>
      </div>

      {#if setupStep === 'idle'}
        <div class='py-6 text-center'>
          <p class='mb-2 text-(lg text-secondary)'>ワンクリックセットアップ</p>
          <p class='mb-4 text-(sm text-tertiary)'>
            上記の処理を自動で実行します。管理者権限が必要です。
          </p>
          <Button
            text='セットアップを開始'
            onclick={oneClickSetup}
            disabled={loading || packageGenerating || setupRunning}
          />
        </div>
      {:else}
        <div class='space-y-4'>
          <!-- プログレス表示 -->
          <div class='rounded bg-(bg-tertiary) p-3'>
            <div class='mb-3 flex items-center gap-3'>
              <div class='text-(sm text-primary) font-medium'>セットアップ進行状況</div>
              {#if setupStep === 'complete'}
                <div class='rounded bg-green-100 px-2 py-1 text-xs text-green-600'>完了</div>
              {:else}
                <div class='rounded bg-blue-100 px-2 py-1 text-xs text-blue-600'>実行中</div>
              {/if}
            </div>

            <div class='space-y-2'>
              <!-- Native Messaging Host セットアップ -->
              <div class='flex items-center gap-3'>
                {#if setupStep === 'host'}
                  <div class='h-4 w-4 animate-spin border-2 border-blue-500 border-t-transparent rounded-full'></div>
                {:else if setupStep === 'complete'}
                  <div class='h-4 w-4 flex items-center justify-center rounded-full bg-green-500'>
                    <svg class='h-2 w-2 text-white' fill='currentColor' viewBox='0 0 20 20'>
                      <path fill-rule='evenodd' d='M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z' clip-rule='evenodd'></path>
                    </svg>
                  </div>
                {:else}
                  <div class='h-4 w-4 border-2 border-gray-300 rounded-full'></div>
                {/if}
                <span class='text-(sm text-primary)'>Native Messaging Host をセットアップ</span>
              </div>
            </div>
          </div>

          <!-- セットアップ出力 -->
          {#if setupOutput}
            <div class='border-border-secondary border rounded bg-(bg-tertiary) p-3'>
              <h4 class='mb-2 text-(sm text-primary) font-medium'>セットアップ出力</h4>
              <pre class='overflow-x-auto whitespace-pre-wrap rounded bg-(bg-primary) p-2 text-(xs text-secondary)'>{setupOutput}</pre>
            </div>
          {/if}

          <!-- アクションボタン -->
          <div class='flex gap-2'>
            {#if setupStep === 'complete'}
              <Button
                variant='normal'
                text='新しいセットアップを開始'
                onclick={() => setupStep = 'idle'}
              />
            {:else}
              <Button
                variant='normal'
                text='セットアップをキャンセル'
                onclick={() => setupStep = 'idle'}
                disabled={setupRunning}
              />
            {/if}

            <Button
              variant='normal'
              text='Native Messaging Host のみセットアップ'
              onclick={setupNativeHost}
              disabled={setupRunning}
            />
          </div>
        </div>
      {/if}
    </div>

    <!-- レジストリキー情報 -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <div class='mb-3 flex items-center justify-between'>
        <h3 class='text-(lg text-primary) font-semibold'>レジストリキー情報</h3>
        <div class='flex gap-2'>
          <Button
            variant='normal'
            text='更新'
            onclick={loadRegistryKeys}
            disabled={registryKeysLoading}
          />
          <Button
            variant='normal'
            text='全て削除'
            onclick={removeRegistryKeys}
            disabled={registryKeysLoading}
          />
        </div>
      </div>

      {#if registryKeysLoading}
        <div class='py-4 text-center'>
          <p class='text-(text-secondary)'>読み込み中...</p>
        </div>
      {:else if registryKeys.length === 0}
        <div class='py-4 text-center'>
          <p class='text-(text-secondary)'>レジストリキー情報がありません</p>
        </div>
      {:else}
        <div class='space-y-3'>
          {#each registryKeys as keyInfo}
            <div class='border-border-secondary border rounded bg-(bg-tertiary) p-3'>
              <div class='mb-2 flex items-center justify-between'>
                <div class='flex items-center gap-2'>
                  <span class='text-(text-primary) font-medium'>{keyInfo.browser}</span>
                  <div class='h-2 w-2 rounded-full' class:bg-green-500={keyInfo.exists} class:bg-red-500={!keyInfo.exists}></div>
                  <span class='text-(xs text-secondary)'>
                    {keyInfo.exists ? '登録済み' : '未登録'}
                  </span>
                </div>
              </div>

              <div class='text-sm space-y-2'>
                <div>
                  <span class='text-(text-secondary)'>レジストリキー:</span>
                  <div class='mt-1 break-all rounded bg-(bg-primary) p-1 text-(xs text-primary) font-mono'>
                    {keyInfo.key_path}
                  </div>
                </div>

                {#if keyInfo.exists && keyInfo.value}
                  <div>
                    <span class='text-(text-secondary)'>マニフェストパス:</span>
                    <div class='mt-1 break-all rounded bg-(bg-primary) p-1 text-(xs text-primary) font-mono'>
                      {keyInfo.value}
                    </div>
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Actions -->
    <div class='flex gap-2'>
      <Button variant='normal' onclick={loadExtensionStatus} text='接続状況を更新' disabled={loading} />
    </div>
  </div>
</div>

<style>
  .toggle {
    position: relative;
    display: inline-flex;
    height: 1.5rem;
    width: 2.75rem;
    align-items: center;
    border-radius: 9999px;
    background-color: rgb(229 231 235);
    transition-property: color, background-color, border-color, text-decoration-color, fill, stroke;
    transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
    transition-duration: 150ms;
  }

  .toggle:focus {
    outline: 2px solid transparent;
    outline-offset: 2px;
    box-shadow: 0 0 0 2px var(--color-accent), 0 0 0 4px rgba(var(--color-accent), 0.1);
  }

  .toggle:checked {
    background-color: var(--color-accent);
  }

  .toggle::before {
    content: '';
    display: inline-block;
    height: 1rem;
    width: 1rem;
    transform: translateX(0.25rem);
    border-radius: 9999px;
    background-color: white;
    transition-property: transform;
    transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
    transition-duration: 150ms;
  }

  .toggle:checked::before {
    transform: translateX(1.5rem);
  }
</style>
