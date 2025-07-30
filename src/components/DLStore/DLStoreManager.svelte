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
    if (!newGameForm.storeId || !newGameForm.erogamescapeId || !newGameForm.purchaseUrl) {
      showErrorToast('すべての項目を入力してください')
      return
    }

    const erogamescapeId = Number.parseInt(newGameForm.erogamescapeId)
    if (Number.isNaN(erogamescapeId)) {
      showErrorToast('有効なErogameScape IDを選択してください')
      return
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
  <div class='space-y-4 max-h-[70vh] overflow-y-auto'>
    <div class='flex items-center justify-between'>
      <h3 class='text-(text-primary lg) font-medium'>未インストールの購入済みゲーム</h3>
      <Button
        leftIcon='i-material-symbols-add-rounded'
        text='新規追加'
        variant='accent'
        onclick={() => (showAddForm = !showAddForm)}
      />
    </div>

    {#if showAddForm}
      <div class='bg-bg-secondary p-4 rounded-lg space-y-3'>
        <h4 class='text-(text-primary base) font-medium'>新しいDL版ゲームを追加</h4>
        <div>
          <label for='store-type' class='block text-(text-secondary sm) mb-1'>ストア</label>
          <select
            id='store-type'
            bind:value={newGameForm.storeType}
            class='w-full p-2 bg-bg-primary border border-border rounded text-text-primary'
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
            <div class='space-y-1 pl-2'>
              <h5 class='text-(text-primary sm) font-medium'>候補</h5>
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
      <div class='text-center py-8'>
        <div class='animate-spin rounded-full h-8 w-8 border-b-2 border-text-primary mx-auto'></div>
      </div>
    {:else if uninstalledGames.length === 0}
      <div class='text-center py-8 text-text-secondary'>
        未インストールの購入済みゲームはありません
      </div>
    {:else}
      <div class='space-y-2'>
        {#each uninstalledGames as game}
          <div class='flex items-center justify-between p-3 bg-bg-secondary rounded-lg'>
            <div class='min-w-0 flex-1'>
              <div class='text-(text-primary base) font-medium truncate'>
                {game.gamename || 'タイトル不明'}
              </div>
              {#if game.dlStore}
                <div class='text-(text-secondary sm)'>
                  {game.dlStore.storeName} - {game.dlStore.storeId}
                </div>
              {/if}
            </div>
            <div class='flex gap-2 ml-4 flex-shrink-0'>
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

    <div class='pt-4 border-t border-border'>
      <p class='text-(text-secondary sm)'>
        ※ Installボタンをクリックすると、ストアページがブラウザで開きます。
        ダウンロード・インストール後、「パスを選択」からゲームの実行ファイルを選択してください。
      </p>
    </div>
  </div>
</Modal>
