<script lang='ts'>
  import type {
    AddTargetForm,
    AddWatchTargetRequest,
    ClearEventsRequest,
    GetEventsForm,
    GetEventsRequest,
    HealthCheckResult,
    ProcTailEvent,
    ProcTailManagerStatus,
    ProcTailVersion,
    RemoveWatchTargetRequest,
    ServiceStatus,
    WatchTarget,
  } from '@/lib/types/proctail'
  import { onMount } from 'svelte'
  import Button from '@/components/UI/Button.svelte'
  import Input from '@/components/UI/Input.svelte'
  import {
    commandProcTailAddWatchTarget,
    commandProcTailClearEvents,
    commandProcTailGetRecordedEvents,
    commandProcTailGetStatus,
    commandProcTailGetWatchTargets,
    commandProcTailHealthCheck,
    commandProcTailIsServiceAvailable,
    commandProcTailManagerDownloadAndInstall,
    commandProcTailManagerGetLatestVersion,
    commandProcTailManagerGetStatus,
    commandProcTailManagerStart,
    commandProcTailManagerStop,
    commandProcTailRemoveWatchTarget,
  } from '@/lib/command'

  // State
  let isServiceAvailable = false
  let serviceStatus: ServiceStatus | null = null
  let watchTargets: WatchTarget[] = []
  let events: ProcTailEvent[] = []
  let healthCheck: HealthCheckResult | null = null
  let error: string | null = null
  let loading = false

  // ProcTail Manager State
  let managerStatus: ProcTailManagerStatus | null = null
  let latestVersion: ProcTailVersion | null = null
  let downloading = false

  // Form data
  let addTargetForm: AddTargetForm = {
    processId: '',
    tag: '',
  }
  let removeTargetTag = ''
  let getEventsForm: GetEventsForm = {
    tag: '',
    count: '50',
    eventType: '',
  }
  let clearEventsTag = ''

  // Initialize
  onMount(async () => {
    await loadManagerStatus()
    await loadLatestVersion()
    await checkServiceAvailability()
    await loadWatchTargets()
    await loadServiceStatus()
    await performHealthCheck()
  })

  // Service availability check
  async function checkServiceAvailability() {
    loading = true
    try {
      isServiceAvailable = await commandProcTailIsServiceAvailable()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `サービス確認エラー: ${errorMessage}`
      isServiceAvailable = false
    }
    finally {
      loading = false
    }
  }

  // Load watch targets
  async function loadWatchTargets() {
    loading = true
    try {
      watchTargets = await commandProcTailGetWatchTargets()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `監視対象取得エラー: ${errorMessage}`
      watchTargets = []
    }
    finally {
      loading = false
    }
  }

  // Load service status
  async function loadServiceStatus() {
    loading = true
    try {
      serviceStatus = await commandProcTailGetStatus()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `サービス状態取得エラー: ${errorMessage}`
      serviceStatus = null
    }
    finally {
      loading = false
    }
  }

  // Perform health check
  async function performHealthCheck() {
    loading = true
    try {
      healthCheck = await commandProcTailHealthCheck()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `ヘルスチェックエラー: ${errorMessage}`
      healthCheck = null
    }
    finally {
      loading = false
    }
  }

  // Add watch target
  async function addWatchTarget() {
    if (!addTargetForm.processId || !addTargetForm.tag) {
      error = 'プロセスIDとタグを入力してください'
      return
    }

    loading = true
    try {
      const request: AddWatchTargetRequest = {
        processId: Number.parseInt(addTargetForm.processId),
        tag: addTargetForm.tag,
      }
      await commandProcTailAddWatchTarget(request)
      addTargetForm = { processId: '', tag: '' }
      await loadWatchTargets()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `監視対象追加エラー: ${errorMessage}`
    }
    finally {
      loading = false
    }
  }

  // Remove watch target
  async function removeWatchTarget() {
    if (!removeTargetTag) {
      error = 'タグを入力してください'
      return
    }

    loading = true
    try {
      const request: RemoveWatchTargetRequest = { tag: removeTargetTag }
      await commandProcTailRemoveWatchTarget(request)
      removeTargetTag = ''
      await loadWatchTargets()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `監視対象削除エラー: ${errorMessage}`
    }
    finally {
      loading = false
    }
  }

  // Get events
  async function getEvents() {
    if (!getEventsForm.tag) {
      error = 'タグを入力してください'
      return
    }

    loading = true
    try {
      const request: GetEventsRequest = {
        tag: getEventsForm.tag,
        count: getEventsForm.count ? Number.parseInt(getEventsForm.count) : undefined,
        eventType: getEventsForm.eventType || undefined,
      }
      events = await commandProcTailGetRecordedEvents(request)
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `イベント取得エラー: ${errorMessage}`
      events = []
    }
    finally {
      loading = false
    }
  }

  // Clear events
  async function clearEvents() {
    if (!clearEventsTag) {
      error = 'タグを入力してください'
      return
    }

    loading = true
    try {
      const request: ClearEventsRequest = { tag: clearEventsTag }
      await commandProcTailClearEvents(request)
      clearEventsTag = ''
      events = []
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `イベントクリアエラー: ${errorMessage}`
    }
    finally {
      loading = false
    }
  }

  // Format event type for display
  function formatEventType(event: ProcTailEvent): string {
    if ('File' in event)
      return 'ファイル操作'
    if ('ProcessStart' in event)
      return 'プロセス開始'
    if ('ProcessEnd' in event)
      return 'プロセス終了'
    return '不明'
  }

  // Get event details
  function getEventDetails(event: ProcTailEvent): string {
    if ('File' in event) {
      return `${event.File.operation}: ${event.File.file_path}`
    }
    if ('ProcessStart' in event) {
      return `${event.ProcessStart.process_name} (PID: ${event.ProcessStart.process_id})`
    }
    if ('ProcessEnd' in event) {
      return `${event.ProcessEnd.process_name} (終了コード: ${event.ProcessEnd.exit_code})`
    }
    return ''
  }

  // Get event timestamp
  function getEventTimestamp(event: ProcTailEvent): string {
    if ('File' in event) {
      return event.File.timestamp
    }
    if ('ProcessStart' in event) {
      return event.ProcessStart.timestamp
    }
    if ('ProcessEnd' in event) {
      return event.ProcessEnd.timestamp
    }
    return ''
  }

  // ProcTail Manager functions
  async function loadManagerStatus() {
    loading = true
    try {
      managerStatus = await commandProcTailManagerGetStatus()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `マネージャー状態取得エラー: ${errorMessage}`
      managerStatus = null
    }
    finally {
      loading = false
    }
  }

  async function loadLatestVersion() {
    loading = true
    try {
      latestVersion = await commandProcTailManagerGetLatestVersion()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `最新バージョン取得エラー: ${errorMessage}`
      latestVersion = null
    }
    finally {
      loading = false
    }
  }

  async function downloadAndInstall() {
    downloading = true
    try {
      await commandProcTailManagerDownloadAndInstall()
      await loadManagerStatus()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `ダウンロード・インストールエラー: ${errorMessage}`
    }
    finally {
      downloading = false
    }
  }

  async function startProcTail() {
    loading = true
    try {
      await commandProcTailManagerStart()
      await loadManagerStatus()
      await checkServiceAvailability()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `ProcTail起動エラー: ${errorMessage}`
    }
    finally {
      loading = false
    }
  }

  async function stopProcTail() {
    loading = true
    try {
      await commandProcTailManagerStop()
      await loadManagerStatus()
      await checkServiceAvailability()
      error = null
    }
    catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error = `ProcTail停止エラー: ${errorMessage}`
    }
    finally {
      loading = false
    }
  }
</script>

<div class='p-6 max-w-4xl mx-auto h-full overflow-y-auto'>
  <div class='space-y-6'>
    <!-- Header -->
    <div class='flex items-center justify-between'>
      <h1 class='text-2xl font-bold text-(text-primary)'>ProcTail デバッグ</h1>
      <div class='flex gap-2'>
        <Button variant='normal' onclick={checkServiceAvailability} text='再接続' />
        <div class='flex items-center gap-2'>
          <div class='w-3 h-3 rounded-full' class:bg-green-500={isServiceAvailable} class:bg-red-500={!isServiceAvailable}></div>
          <span class='text-sm text-(text-secondary)'>{isServiceAvailable ? '接続済み' : '未接続'}</span>
        </div>
      </div>
    </div>

    <!-- Error Display -->
    {#if error}
      <div class='bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded'>
        {error}
      </div>
    {/if}

    <!-- ProcTail Manager Status -->
    {#if managerStatus}
      <div class='bg-(bg-secondary) border border-(border-primary) rounded p-4'>
        <h3 class='text-lg font-semibold mb-3 text-(text-primary)'>ProcTail マネージャー</h3>
        <div class='grid grid-cols-2 gap-4 text-sm mb-4'>
          <div>
            <span class='font-medium text-(text-secondary)'>現在のバージョン:</span>
            <span class='text-(text-primary)'>{managerStatus.current_version || '未インストール'}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>実行可能ファイル:</span>
            <span class='text-(text-primary)'>{managerStatus.executable_exists ? '存在' : '不存在'}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>プロセス状態:</span>
            <span class='text-(text-primary)'>{managerStatus.is_running ? '実行中' : '停止中'}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>アップデート:</span>
            <span class='text-(text-primary)'>{managerStatus.update_available ? '利用可能' : '最新'}</span>
          </div>
        </div>

        {#if latestVersion}
          <div class='mb-4'>
            <span class='font-medium text-(text-secondary)'>最新バージョン:</span>
            <span class='text-(text-primary)'>{latestVersion.version}</span>
          </div>
        {/if}

        <div class='flex gap-2 flex-wrap'>
          {#if managerStatus.update_available}
            <Button
              onclick={downloadAndInstall}
              text={downloading ? 'ダウンロード中...' : 'ダウンロード・インストール'}
              disabled={downloading || loading}
            />
          {/if}

          {#if managerStatus.executable_exists}
            {#if managerStatus.is_running}
              <Button variant='normal' onclick={stopProcTail} text='ProcTail停止' disabled={loading} />
            {:else}
              <Button onclick={startProcTail} text='ProcTail起動' disabled={loading} />
            {/if}
          {/if}

          <Button variant='normal' onclick={loadManagerStatus} text='状態更新' disabled={loading} />
        </div>
      </div>
    {/if}

    <!-- Service Status -->
    {#if serviceStatus}
      <div class='bg-(bg-secondary) border border-(border-primary) rounded p-4'>
        <h3 class='text-lg font-semibold mb-3 text-(text-primary)'>サービス状態</h3>
        <div class='grid grid-cols-2 gap-4 text-sm'>
          <div>
            <span class='font-medium text-(text-secondary)'>状態:</span>
            <span class='text-(text-primary)'>{serviceStatus.service.status}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>バージョン:</span>
            <span class='text-(text-primary)'>{serviceStatus.service.version}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>起動時間:</span>
            <span class='text-(text-primary)'>{serviceStatus.service.start_time}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>稼働時間:</span>
            <span class='text-(text-primary)'>{serviceStatus.service.uptime}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>アクティブタグ:</span>
            <span class='text-(text-primary)'>{serviceStatus.monitoring.active_tags}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>監視プロセス:</span>
            <span class='text-(text-primary)'>{serviceStatus.monitoring.active_processes}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>総イベント数:</span>
            <span class='text-(text-primary)'>{serviceStatus.monitoring.total_events}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>メモリ使用量:</span>
            <span class='text-(text-primary)'>{serviceStatus.resources.memory_usage_mb.toFixed(1)} MB</span>
          </div>
        </div>
      </div>
    {/if}

    <!-- Health Check -->
    {#if healthCheck}
      <div class='bg-(bg-secondary) border border-(border-primary) rounded p-4'>
        <h3 class='text-lg font-semibold mb-3 text-(text-primary)'>ヘルスチェック</h3>
        <div class='space-y-2'>
          <div>
            <span class='font-medium text-(text-secondary)'>状態:</span>
            <span class='text-(text-primary)'>{healthCheck.status}</span>
          </div>
          <div>
            <span class='font-medium text-(text-secondary)'>チェック時間:</span>
            <span class='text-(text-primary)'>{healthCheck.check_time}</span>
          </div>
          <div class='mt-3'>
            <span class='font-medium text-(text-secondary)'>詳細:</span>
            <div class='mt-1 space-y-1'>
              {#each Object.entries(healthCheck.details) as [key, value]}
                <div class='flex justify-between'>
                  <span class='text-(text-secondary)'>{key}:</span>
                  <span class='text-(text-primary)'>{value}</span>
                </div>
              {/each}
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- Watch Targets -->
    <div class='bg-(bg-secondary) border border-(border-primary) rounded p-4'>
      <h3 class='text-lg font-semibold mb-3 text-(text-primary)'>監視対象</h3>

      <!-- Add Watch Target -->
      <div class='mb-4 p-3 bg-(bg-tertiary) rounded'>
        <h4 class='font-medium mb-2 text-(text-secondary)'>監視対象を追加</h4>
        <div class='flex gap-2'>
          <Input bind:value={addTargetForm.processId} placeholder='プロセスID' />
          <Input bind:value={addTargetForm.tag} placeholder='タグ' />
          <Button onclick={addWatchTarget} text='追加' disabled={loading} />
        </div>
      </div>

      <!-- Remove Watch Target -->
      <div class='mb-4 p-3 bg-(bg-tertiary) rounded'>
        <h4 class='font-medium mb-2 text-(text-secondary)'>監視対象を削除</h4>
        <div class='flex gap-2'>
          <Input bind:value={removeTargetTag} placeholder='タグ' />
          <Button onclick={removeWatchTarget} text='削除' disabled={loading} />
        </div>
      </div>

      <!-- Watch Targets List -->
      {#if watchTargets.length > 0}
        <div class='space-y-2'>
          {#each watchTargets as target}
            <div class='flex items-center justify-between p-2 bg-(bg-tertiary) rounded'>
              <div>
                <span class='font-medium text-(text-primary)'>{target.tag}</span>
                <span class='text-(text-secondary) ml-2'>({target.process_name} - PID: {target.process_id})</span>
              </div>
              <div class='flex items-center gap-2'>
                <div class='w-2 h-2 rounded-full' class:bg-green-500={target.is_running} class:bg-red-500={!target.is_running}></div>
                <span class='text-sm text-(text-secondary)'>{target.is_running ? '実行中' : '停止'}</span>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <p class='text-(text-secondary)'>監視対象がありません</p>
      {/if}

      <div class='mt-4'>
        <Button variant='normal' onclick={loadWatchTargets} text='更新' disabled={loading} />
      </div>
    </div>

    <!-- Events -->
    <div class='bg-(bg-secondary) border border-(border-primary) rounded p-4'>
      <h3 class='text-lg font-semibold mb-3 text-(text-primary)'>イベント</h3>

      <!-- Get Events -->
      <div class='mb-4 p-3 bg-(bg-tertiary) rounded'>
        <h4 class='font-medium mb-2 text-(text-secondary)'>イベントを取得</h4>
        <div class='flex gap-2'>
          <Input bind:value={getEventsForm.tag} placeholder='タグ' />
          <Input bind:value={getEventsForm.count} placeholder='件数 (デフォルト: 50)' />
          <Input bind:value={getEventsForm.eventType} placeholder='イベントタイプ (オプション)' />
          <Button onclick={getEvents} text='取得' disabled={loading} />
        </div>
      </div>

      <!-- Clear Events -->
      <div class='mb-4 p-3 bg-(bg-tertiary) rounded'>
        <h4 class='font-medium mb-2 text-(text-secondary)'>イベントをクリア</h4>
        <div class='flex gap-2'>
          <Input bind:value={clearEventsTag} placeholder='タグ' />
          <Button onclick={clearEvents} text='クリア' disabled={loading} />
        </div>
      </div>

      <!-- Events List -->
      {#if events.length > 0}
        <div class='space-y-2 max-h-96 overflow-y-auto'>
          {#each events as event}
            <div class='p-2 bg-(bg-tertiary) rounded'>
              <div class='flex items-center justify-between'>
                <span class='font-medium text-(text-primary)'>{formatEventType(event)}</span>
                <span class='text-sm text-(text-secondary)'>{getEventTimestamp(event)}</span>
              </div>
              <div class='text-sm text-(text-secondary) mt-1'>
                {getEventDetails(event)}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <p class='text-(text-secondary)'>イベントがありません</p>
      {/if}
    </div>

    <!-- Actions -->
    <div class='flex gap-2'>
      <Button variant='normal' onclick={loadServiceStatus} text='サービス状態更新' disabled={loading} />
      <Button variant='normal' onclick={performHealthCheck} text='ヘルスチェック実行' disabled={loading} />
    </div>
  </div>
</div>
