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
  let isServiceAvailable = $state(false)
  let serviceStatus = $state<ServiceStatus | null>(null)
  let watchTargets = $state<WatchTarget[]>([])
  let events = $state<ProcTailEvent[]>([])
  let healthCheck = $state<HealthCheckResult | null>(null)
  let error = $state<string | null>(null)
  let loading = $state(false)

  // ProcTail Manager State
  let managerStatus = $state<ProcTailManagerStatus | null>(null)
  let latestVersion = $state<ProcTailVersion | null>(null)
  let downloading = $state(false)

  // Form data
  let addTargetForm = $state<AddTargetForm>({
    processId: '',
    tag: '',
  })
  let removeTargetTag = $state('')
  let getEventsForm = $state<GetEventsForm>({
    tag: '',
    count: '50',
    eventType: '',
  })
  let clearEventsTag = $state('')

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
      return `${event.File.Operation}: ${event.File.FilePath}`
    }
    if ('ProcessStart' in event) {
      return `${event.ProcessStart.ProcessName} (PID: ${event.ProcessStart.ProcessId})`
    }
    if ('ProcessEnd' in event) {
      return `${event.ProcessEnd.ProcessName} (終了コード: ${event.ProcessEnd.ExitCode})`
    }
    return ''
  }

  // Get event timestamp
  function getEventTimestamp(event: ProcTailEvent): string {
    if ('File' in event) {
      return event.File.Timestamp
    }
    if ('ProcessStart' in event) {
      return event.ProcessStart.Timestamp
    }
    if ('ProcessEnd' in event) {
      return event.ProcessEnd.Timestamp
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
      await loadLatestVersion()
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

<div class='mx-auto h-full max-w-4xl overflow-y-auto p-6'>
  <div class='space-y-6'>
    <!-- Header -->
    <div class='flex items-center justify-between'>
      <h1 class='text-(2xl text-primary) font-bold'>ProcTail デバッグ</h1>
      <div class='flex gap-2'>
        <Button variant='normal' onclick={checkServiceAvailability} text='再接続' />
        <div class='flex items-center gap-2'>
          <div class='h-3 w-3 rounded-full' class:bg-green-500={isServiceAvailable} class:bg-red-500={!isServiceAvailable}></div>
          <span class='text-(sm text-secondary)'>{isServiceAvailable ? '接続済み' : '未接続'}</span>
        </div>
      </div>
    </div>

    <!-- Error Display -->
    {#if error}
      <div class='border border-red-400 rounded bg-red-100 px-4 py-3 text-red-700'>
        {error}
      </div>
    {/if}

    <!-- ProcTail Manager Status -->
    {#if managerStatus}
      <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
        <h3 class='mb-3 text-(lg text-primary) font-semibold'>ProcTail マネージャー</h3>
        <div class='grid grid-cols-2 mb-4 gap-4 text-sm'>
          <div>
            <span class='text-(text-secondary) font-medium'>現在のバージョン:</span>
            <span class='text-(text-primary)'>{managerStatus.current_version || '未インストール'}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>実行可能ファイル:</span>
            <span class='text-(text-primary)'>{managerStatus.executable_exists ? '存在' : '不存在'}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>プロセス状態:</span>
            <span class='text-(text-primary)'>{managerStatus.is_running ? '実行中' : '停止中'}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>アップデート:</span>
            <span class='text-(text-primary)'>{managerStatus.update_available ? '利用可能' : '最新'}</span>
          </div>
        </div>

        {#if latestVersion}
          <div class='mb-4'>
            <span class='text-(text-secondary) font-medium'>最新バージョン:</span>
            <span class='text-(text-primary)'>{latestVersion.version}</span>
          </div>
        {/if}

        <div class='flex flex-wrap gap-2'>
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
      <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
        <h3 class='mb-3 text-(lg text-primary) font-semibold'>サービス状態</h3>
        <div class='grid grid-cols-2 gap-4 text-sm'>
          <div>
            <span class='text-(text-secondary) font-medium'>状態:</span>
            <span class='text-(text-primary)'>{serviceStatus.Service.Status}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>バージョン:</span>
            <span class='text-(text-primary)'>{serviceStatus.Service.Version}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>起動時間:</span>
            <span class='text-(text-primary)'>{serviceStatus.Service.StartTime}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>稼働時間:</span>
            <span class='text-(text-primary)'>{serviceStatus.Service.Uptime}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>アクティブタグ:</span>
            <span class='text-(text-primary)'>{serviceStatus.Monitoring.ActiveTags}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>監視プロセス:</span>
            <span class='text-(text-primary)'>{serviceStatus.Monitoring.ActiveProcesses}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>総イベント数:</span>
            <span class='text-(text-primary)'>{serviceStatus.Monitoring.TotalEvents}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>メモリ使用量:</span>
            <span class='text-(text-primary)'>{serviceStatus.Resources.MemoryUsageMB.toFixed(1)} MB</span>
          </div>
        </div>
      </div>
    {/if}

    <!-- Health Check -->
    {#if healthCheck}
      <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
        <h3 class='mb-3 text-(lg text-primary) font-semibold'>ヘルスチェック</h3>
        <div class='space-y-2'>
          <div>
            <span class='text-(text-secondary) font-medium'>状態:</span>
            <span class='text-(text-primary)'>{healthCheck.Status}</span>
          </div>
          <div>
            <span class='text-(text-secondary) font-medium'>チェック時間:</span>
            <span class='text-(text-primary)'>{healthCheck.CheckTime}</span>
          </div>
          <div class='mt-3'>
            <span class='text-(text-secondary) font-medium'>詳細:</span>
            <div class='mt-1 space-y-1'>
              {#each Object.entries(healthCheck.Details) as [key, value]}
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
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <h3 class='mb-3 text-(lg text-primary) font-semibold'>監視対象</h3>

      <!-- Add Watch Target -->
      <div class='mb-4 rounded bg-(bg-tertiary) p-3'>
        <h4 class='mb-2 text-(text-secondary) font-medium'>監視対象を追加</h4>
        <div class='grid grid-cols-1 gap-2 md:grid-cols-3'>
          <Input bind:value={addTargetForm.processId} placeholder='プロセスID (例: 1234)' />
          <Input bind:value={addTargetForm.tag} placeholder='タグ (例: game-process)' />
          <Button onclick={addWatchTarget} text='追加' disabled={loading} />
        </div>
        <div class='mt-2 text-(xs text-tertiary)'>
          プロセスIDはタスクマネージャーで確認できます
        </div>
      </div>

      <!-- Remove Watch Target -->
      <div class='mb-4 rounded bg-(bg-tertiary) p-3'>
        <h4 class='mb-2 text-(text-secondary) font-medium'>監視対象を削除</h4>
        <div class='grid grid-cols-1 gap-2 md:grid-cols-2'>
          <Input bind:value={removeTargetTag} placeholder='削除するタグ名' />
          <Button onclick={removeWatchTarget} text='削除' disabled={loading} variant='error' />
        </div>
      </div>

      <!-- Watch Targets List -->
      {#if watchTargets.length > 0}
        <div class='space-y-2'>
          {#each watchTargets as target}
            <div class='border-border-secondary flex items-center justify-between border rounded bg-(bg-tertiary) p-3'>
              <div class='flex flex-col gap-2'>
                <div class='flex items-center justify-between'>
                  <div class='flex items-center gap-2'>
                    <span class='text-(xs text-secondary) font-medium'>タグ:</span>
                    <span class='rounded bg-(bg-primary) px-3 py-1 text-(sm bg-secondary text-primary) font-semibold font-mono'>{target.Tag}</span>
                  </div>
                  <div class='flex items-center gap-1'>
                    <div class='h-3 w-3 rounded-full' class:bg-green-500={target.IsRunning} class:bg-red-500={!target.IsRunning}></div>
                    <span class='text-sm font-medium' class:text-green-700={target.IsRunning} class:text-red-700={!target.IsRunning}>
                      {target.IsRunning ? '実行中' : '停止'}
                    </span>
                  </div>
                </div>
                <div class='grid grid-cols-1 gap-3 text-sm md:grid-cols-3'>
                  <div class='flex flex-col gap-1'>
                    <span class='text-(xs text-secondary)'>プロセス名</span>
                    <span class='text-(text-primary) font-medium'>{target.ProcessName || '(不明)'}</span>
                  </div>
                  <div class='flex flex-col gap-1'>
                    <span class='text-(xs text-secondary)'>プロセスID</span>
                    <span class='text-(text-primary) font-mono'>{target.ProcessId || 'N/A'}</span>
                  </div>
                  <div class='flex flex-col gap-1'>
                    <span class='text-(xs text-secondary)'>開始時刻</span>
                    <span class='text-(xs text-primary)'>
                      {target.StartTime && !Number.isNaN(new Date(target.StartTime).getTime())
                        ? new Date(target.StartTime).toLocaleString()
                        : 'N/A'}
                    </span>
                  </div>
                </div>
                {#if !target.IsRunning}
                  <div class='mt-2 rounded bg-orange-100 px-2 py-1 text-(xs orange-800 text-tertiary)'>
                    ⚠️ このプロセスは終了していますが、監視対象として登録されたままです。イベント履歴は保持されています。
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class='py-8 text-center'>
          <p class='text-(lg text-secondary)'>監視対象がありません</p>
          <p class='mt-1 text-(sm text-tertiary)'>上記のフォームからプロセスを追加してください</p>
        </div>
      {/if}

      <div class='mt-4'>
        <Button variant='normal' onclick={loadWatchTargets} text='更新' disabled={loading} />
      </div>
    </div>

    <!-- Events -->
    <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
      <h3 class='mb-3 text-(lg text-primary) font-semibold'>イベント</h3>

      <!-- Get Events -->
      <div class='mb-4 rounded bg-(bg-tertiary) p-3'>
        <h4 class='mb-2 text-(text-secondary) font-medium'>イベントを取得</h4>
        <div class='grid grid-cols-1 gap-2 md:grid-cols-4'>
          <Input bind:value={getEventsForm.tag} placeholder='タグ' />
          <Input bind:value={getEventsForm.count} placeholder='件数 (50)' />
          <Input bind:value={getEventsForm.eventType} placeholder='イベントタイプ' />
          <Button onclick={getEvents} text='取得' disabled={loading} />
        </div>
        <div class='mt-2 text-(xs text-tertiary)'>
          イベントタイプ: File, ProcessStart, ProcessEnd など
        </div>
      </div>

      <!-- Clear Events -->
      <div class='mb-4 rounded bg-(bg-tertiary) p-3'>
        <h4 class='mb-2 text-(text-secondary) font-medium'>イベントをクリア</h4>
        <div class='grid grid-cols-1 gap-2 md:grid-cols-2'>
          <Input bind:value={clearEventsTag} placeholder='クリアするタグ名' />
          <Button onclick={clearEvents} text='クリア' disabled={loading} variant='error' />
        </div>
        <div class='mt-2 text-(xs text-tertiary)'>
          指定したタグのすべてのイベントが削除されます
        </div>
      </div>

      <!-- Events List -->
      {#if events.length > 0}
        <div class='border-border-secondary max-h-96 overflow-y-auto border rounded p-2 space-y-2'>
          <div class='mb-2 px-2 text-(sm text-secondary)'>
            {events.length}件のイベントが見つかりました
          </div>
          {#each events as event, index}
            <div class='border-border-secondary border rounded bg-(bg-tertiary) p-3 transition-colors hover:bg-(bg-secondary)'>
              <div class='mb-2 flex items-center justify-between'>
                <div class='flex items-center gap-2'>
                  <span class='text-(xs text-tertiary) font-mono'>#{events.length - index}</span>
                  <span class='rounded px-2 py-1 text-(xs text-primary) font-medium'
                        class:bg-blue-100={formatEventType(event) === 'ファイル操作'}
                        class:bg-green-100={formatEventType(event) === 'プロセス開始'}
                        class:bg-red-100={formatEventType(event) === 'プロセス終了'}
                        class:text-blue-800={formatEventType(event) === 'ファイル操作'}
                        class:text-green-800={formatEventType(event) === 'プロセス開始'}
                        class:text-red-800={formatEventType(event) === 'プロセス終了'}>
                    {formatEventType(event)}
                  </span>
                </div>
                <span class='text-(xs text-secondary) font-mono'>
                  {new Date(getEventTimestamp(event)).toLocaleString()}
                </span>
              </div>
              <div class='bg-bg-quaternary break-all rounded p-2 text-(wrap sm text-primary) font-mono'>
                {getEventDetails(event)}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class='border-border-secondary border rounded bg-(bg-tertiary) py-8 text-center'>
          <p class='text-(lg text-secondary)'>イベントがありません</p>
          <p class='mt-1 text-(sm text-tertiary)'>上記のフォームからイベントを取得してください</p>
        </div>
      {/if}
    </div>

    <!-- Actions -->
    <div class='flex gap-2'>
      <Button variant='normal' onclick={loadServiceStatus} text='サービス状態更新' disabled={loading} />
      <Button variant='normal' onclick={performHealthCheck} text='ヘルスチェック実行' disabled={loading} />
    </div>
  </div>
</div>
