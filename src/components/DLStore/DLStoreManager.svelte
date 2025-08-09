<script lang='ts'>
  import type { CollectionElement } from '@/lib/types'
  import { open } from '@tauri-apps/plugin-dialog'
  import Button from '@/components/UI/Button.svelte'
  import Input from '@/components/UI/Input.svelte'
  import Modal from '@/components/UI/Modal.svelte'
  import {
    commandGetGameCandidatesByName,
    commandGetUninstalledOwnedGames,
    commandLinkInstalledGame,
    commandOpenStorePage,
    commandRegisterDLStoreGame,
  } from '@/lib/command'
  import { showErrorToast, showInfoToast } from '@/lib/toast'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'

  interface Props {
    isOpen: boolean
  }

  let { isOpen = $bindable() }: Props = $props()

  let uninstalledGames = $state<CollectionElement[]>([])
  let loading = $state(false)
  let showAddForm = $state(false)

  // 新規ゲーム登録フォーム
  let newGameForm = $state({
    storeType: 'DMM' as 'DMM' | 'DLSite',
    storeId: '',
    gameTitle: '',
    purchaseUrl: '',
    erogamescapeId: '',
  })

  // ゲーム候補
  let candidates = $state<[number, string][]>([])

  // debounce用のタイマー
  let debounceTimer: ReturnType<typeof setTimeout> | undefined

  // ゲーム名が変更された時の候補取得（debounce付き）
  const updateCandidates = (gameTitle: string) => {
    // 既存のタイマーをクリア
    if (debounceTimer !== undefined) {
      clearTimeout(debounceTimer)
    }

    if (!gameTitle.trim()) {
      candidates = []
      return
    }

    // 300ms待ってから実行
    debounceTimer = setTimeout(async () => {
      try {
        candidates = await commandGetGameCandidatesByName(gameTitle)
      }
      catch (e) {
        console.error('Error fetching candidates:', e)
        showErrorToast(`候補の取得に失敗しました: ${e}`)
        candidates = []
      }
    }, 300)
  }

  // 候補をクリックした時
  const selectCandidate = (id: number) => {
    newGameForm.erogamescapeId = `${id}`
  }

  const loadUninstalledGames = async () => {
    loading = true
    try {
      uninstalledGames = await commandGetUninstalledOwnedGames()
    }
    catch (e) {
      showErrorToast(`Failed to load games: ${e}`)
    }
    finally {
      loading = false
    }
  }

  const registerNewGame = async () => {
    if (!newGameForm.storeId || !newGameForm.purchaseUrl) {
      showErrorToast('ストアIDと購入URLは必須です')
      return
    }

    let erogamescapeId: number | null = null
    if (newGameForm.erogamescapeId && newGameForm.erogamescapeId.trim() !== '') {
      const parsed = Number.parseInt(newGameForm.erogamescapeId)
      if (Number.isNaN(parsed)) {
        showErrorToast('ErogameScape IDが不正です')
        return
      }
      erogamescapeId = parsed
    }

    try {
      await commandRegisterDLStoreGame(
        newGameForm.storeType,
        newGameForm.storeId,
        erogamescapeId,
        newGameForm.purchaseUrl,
      )
      showInfoToast('ゲームを登録しました')

      // フォームをリセット
      newGameForm = {
        storeType: 'DMM',
        storeId: '',
        gameTitle: '',
        purchaseUrl: '',
        erogamescapeId: '',
      }
      candidates = []
      showAddForm = false

      // リストを再読み込み
      await loadUninstalledGames()
      await sidebarCollectionElements.refetch()
    }
    catch (e) {
      showErrorToast(`Failed to register game: ${e}`)
    }
  }

  const openStorePage = async (purchaseUrl: string) => {
    try {
      await commandOpenStorePage(purchaseUrl)
    }
    catch (e) {
      showErrorToast(`Failed to open store page: ${e}`)
    }
  }

  const linkInstalledGame = async (collectionElementId: number) => {
    try {
      const selected = await open({
        filters: [{ name: 'Executable', extensions: ['exe'] }],
      })

      if (selected) {
        await commandLinkInstalledGame(collectionElementId, selected)
        showInfoToast('ゲームのパスを関連付けました')
        await loadUninstalledGames()
        await sidebarCollectionElements.refetch()
      }
    }
    catch (e) {
      showErrorToast(`Failed to link game: ${e}`)
    }
  }

  $effect(() => {
    if (isOpen) {
      loadUninstalledGames()
    }
  })
</script>

<Modal
  {isOpen}
  onclose={() => (isOpen = false)}
  title='DL版ゲーム管理'
  cancelText='閉じる'
  oncancel={() => (isOpen = false)}
>
  <div class='max-h-[70vh] overflow-y-auto space-y-4'>
    <div class='flex items-center justify-between'>
      <h3 class='text-(lg text-primary) font-medium'>未インストールの購入済みゲーム</h3>
      <Button
        leftIcon='i-material-symbols-add-rounded'
        text='新規追加'
        variant='accent'
        onclick={() => (showAddForm = !showAddForm)}
      />
    </div>

    {#if showAddForm}
      <div class='rounded-lg bg-bg-secondary p-4 space-y-3'>
        <h4 class='text-(base text-primary) font-medium'>新しいDL版ゲームを追加</h4>
        <div>
          <label for='store-type' class='mb-1 block text-(sm text-secondary)'>ストア</label>
          <select
            id='store-type'
            bind:value={newGameForm.storeType}
            class='border-border w-full border rounded bg-bg-primary p-2 text-text-primary'
          >
            <option value='DMM'>DMM Games</option>
            <option value='DLSite'>DLsite</option>
          </select>
        </div>
        <Input
          label='ストアID'
          bind:value={newGameForm.storeId}
          placeholder='例: 123456'
        />
        <div class='space-y-1'>
          <Input
            label='ゲームタイトル（検索用）'
            bind:value={newGameForm.gameTitle}
            placeholder='ゲーム名を入力'
            on:update={e => updateCandidates(e.detail.value)}
          />
          {#if candidates.length > 0}
            <div class='pl-2 space-y-1'>
              <h5 class='text-(sm text-primary) font-medium'>候補</h5>
              <div class='max-h-32 overflow-y-auto space-y-1'>
                {#each candidates as [id, gamename] (id)}
                  <button
                    class={`rounded px-3 py-1 text-left hover:bg-bg-button-hover transition-all block w-full text-(text-primary sm) ${
                      newGameForm.erogamescapeId === `${id}` ? 'bg-bg-button' : 'bg-transparent'
                    }`}
                    onclick={() => selectCandidate(id)}
                  >
                    <div class='truncate'>
                      {gamename}
                    </div>
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>
        <Input
          label='ErogameScape ID'
          bind:value={newGameForm.erogamescapeId}
          placeholder='候補から選択するか直接入力'
          readonly={candidates.length > 0}
        />
        <Input
          label='購入ページURL'
          bind:value={newGameForm.purchaseUrl}
          placeholder='https://...'
        />
        <div class='flex gap-2'>
          <Button
            text='登録'
            variant='accent'
            onclick={registerNewGame}
          />
          <Button
            text='キャンセル'
            variant='normal'
            onclick={() => (showAddForm = false)}
          />
        </div>
      </div>
    {/if}

    {#if loading}
      <div class='py-8 text-center'>
        <div class='mx-auto h-8 w-8 animate-spin border-b-2 border-text-primary rounded-full'></div>
      </div>
    {:else if uninstalledGames.length === 0}
      <div class='py-8 text-center text-text-secondary'>
        未インストールの購入済みゲームはありません
      </div>
    {:else}
      <div class='space-y-2'>
        {#each uninstalledGames as game}
          <div class='flex items-center justify-between rounded-lg bg-bg-secondary p-3'>
            <div class='min-w-0 flex-1'>
              <div class='truncate text-(base text-primary) font-medium'>
                {game.gamename || 'タイトル不明'}
              </div>
              {#if game.dlStore}
                <div class='text-(sm text-secondary)'>
                  {game.dlStore.storeName} - {game.dlStore.storeId}
                </div>
              {/if}
            </div>
            <div class='ml-4 flex flex-shrink-0 gap-2'>
              <Button
                leftIcon='i-material-symbols-download-rounded'
                text='Install'
                variant='accent'
                onclick={() => game.dlStore && openStorePage(game.dlStore.purchaseUrl)}
              />
              <Button
                leftIcon='i-material-symbols-folder-open'
                text='パスを選択'
                variant='normal'
                onclick={() => linkInstalledGame(game.id)}
              />
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <div class='border-border border-t pt-4'>
      <p class='text-(sm text-secondary)'>
        ※ Installボタンをクリックすると、ストアページがブラウザで開きます。
        ダウンロード・インストール後、「パスを選択」からゲームの実行ファイルを選択してください。
      </p>
    </div>
  </div>
</Modal>
