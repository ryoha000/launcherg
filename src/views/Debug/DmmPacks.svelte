<script lang='ts'>
  import { invoke } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'
  import Button from '@/components/UI/Button.svelte'

  type Element = {
    id: number
    gamename: string
    dmm?: { id: number, collectionElementId: number, category: string, subcategory: string } | null
  }

  let elements: Element[] = []
  let packList: Array<{ id: number, storeId: string }> = []
  let newStoreId = ''
  let loading = false

  async function load() {
    loading = true
    try {
      const all = await invoke<Element[]>('get_all_elements')
      elements = all
      const packs = await invoke<Array<{ id: number, storeId: string }>>('dmm_pack_all')
      packList = packs
    }
    finally {
      loading = false
    }
  }

  async function addPackManually() {
    const sid = newStoreId.trim()
    if (!sid)
      return
    await invoke('dmm_pack_add', { storeId: sid })
    newStoreId = ''
    await load()
  }

  async function removePack(id: number, storeId: string) {
    await invoke('dmm_pack_remove', { storeId })
    await load()
  }

  onMount(load)
</script>

<div class='mx-auto h-full max-w-3xl overflow-y-auto p-6 space-y-4'>
  <h2 class='text-(lg text-primary) font-bold'>DMM パック管理</h2>
  {#if loading}
    <div class='text-(text-secondary)'>Loading...</div>
  {:else}
    <div class='overflow-hidden border border-(border-primary) rounded bg-(bg-secondary)'>
      <table class='w-full text-(sm text-primary)'>
        <thead class='text-(sm text-secondary)'>
          <tr>
            <th class='px-3 py-2 text-left'>名前</th>
            <th class='px-3 py-2 text-left'>DMM</th>
            <th class='px-3 py-2 text-left'>Pack</th>
          </tr>
        </thead>
        <tbody>
          {#each elements as el}
            {#if el.dmm}
              <tr class='border-(t border-primary)'>
                <td class='px-3 py-2'>{el.gamename}</td>
                <td class='px-3 py-2'>{el.dmm?.category}/{el.dmm?.subcategory}</td>
                <td class='px-3 py-2'>
                  <input type='checkbox' disabled title='storeId 取得未対応' />
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>

    <div class='mt-6 space-y-2'>
      <h3 class='text-(sm text-secondary) font-semibold'>登録済み Pack</h3>
      <div class='border border-(border-primary) rounded bg-(bg-secondary)'>
        {#if packList.length === 0}
          <div class='p-6 text-(center text-secondary)'>なし</div>
        {:else}
          <div class='divide-(y border-primary)'>
            {#each packList as p}
              <div class='flex items-center justify-between p-3'>
                <div class='border border-(border-primary) rounded bg-(bg-primary) px-1 py-0.5 text-(sm text-primary) font-mono'>
                  {p.storeId}
                </div>
                <Button text='削除' onclick={() => removePack(p.id, p.storeId)} />
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class='mt-2 flex items-end gap-3'>
        <div class='flex-1'>
          <div class='mb-1 text-(sm text-secondary)'>Store ID</div>
          <input class='w-full border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)' placeholder='storeId を入力 (例: purple_0028pack)' bind:value={newStoreId} />
        </div>
        <Button text='追加' onclick={addPackManually} disabled={loading} />
      </div>
    </div>
  {/if}
</div>
