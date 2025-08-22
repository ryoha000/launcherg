<script lang='ts'>
  import { invoke } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'

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

<div class='p-4 space-y-4'>
  <h2 class='text-lg font-bold'>DMM パック管理</h2>
  {#if loading}
    <div>Loading...</div>
  {:else}
    <table class='w-full text-sm'>
      <thead>
        <tr><th class='text-left'>名前</th><th>DMM</th><th>Pack</th></tr>
      </thead>
      <tbody>
        {#each elements as el}
          {#if el.dmm}
            <tr>
              <td>{el.gamename}</td>
              <td>{el.dmm?.category}/{el.dmm?.subcategory}</td>
              <td>
                <input type='checkbox' disabled title='storeId 取得未対応' />
              </td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>

    <div class='mt-6 space-y-2'>
      <h3 class='font-semibold'>登録済み Pack</h3>
      <ul class='space-y-1'>
        {#each packList as p}
          <li class='flex items-center gap-2'>
            <code class='rounded bg-gray-100 px-1 py-0.5'>{p.storeId}</code>
            <button class='border rounded px-2 py-1' on:click={() => removePack(p.id, p.storeId)}>削除</button>
          </li>
        {/each}
        {#if packList.length === 0}
          <li class='text-gray-500'>なし</li>
        {/if}
      </ul>

      <div class='mt-2 flex items-center gap-2'>
        <input class='w-64 border rounded px-2 py-1' placeholder='storeId を入力 (例: purple_0028pack)' bind:value={newStoreId} />
        <button class='border rounded px-2 py-1' on:click={addPackManually}>追加</button>
      </div>
    </div>
  {/if}
</div>
