<script lang='ts'>
  import type { DenyListItemVm } from '@/lib/command'
  import { onMount } from 'svelte'
  import Button from '@/components/UI/Button.svelte'
  import { commandDenyListAdd, commandDenyListAll, commandDenyListRemove } from '@/lib/command'

  const STORE_TYPES = [
    { value: 1, label: 'DMM' },
    { value: 2, label: 'DLsite' },
  ]

  let items = $state<DenyListItemVm[]>([])
  let storeType = $state(1)
  let storeId = $state('')
  let loading = $state(false)

  const reload = async () => {
    loading = true
    try {
      items = await commandDenyListAll()
    }
    finally {
      loading = false
    }
  }

  const add = async () => {
    const id = storeId.trim()
    if (!id)
      return
    await commandDenyListAdd(storeType, id)
    storeId = ''
    await reload()
  }

  const remove = async (it: DenyListItemVm) => {
    await commandDenyListRemove(it.storeType, it.storeId)
    await reload()
  }

  onMount(reload)
</script>

<div class='mx-auto h-full max-w-3xl overflow-y-auto p-6'>
  <div class='mb-4 flex items-end gap-3'>
    <div>
      <div class='mb-1 text-(sm text-secondary)'>ストア</div>
      <select bind:value={storeType} class='border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)'>
        {#each STORE_TYPES as st}
          <option value={st.value}>{st.label}</option>
        {/each}
      </select>
    </div>
    <div class='flex-1'>
      <div class='mb-1 text-(sm text-secondary)'>Store ID</div>
      <input bind:value={storeId} class='w-full border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)' placeholder='例: dmm: product id / dlsite: RJxxxxxx など' />
    </div>
    <Button text='追加' onclick={add} disabled={loading} />
  </div>

  <div class='border border-(border-primary) rounded bg-(bg-secondary)'>
    {#if items.length === 0}
      <div class='p-6 text-(center text-secondary)'>登録がありません</div>
    {:else}
      <div class='divide-(y border-primary)'>
        {#each items as it}
          <div class='flex items-center justify-between p-3'>
            <div class='text-(sm text-primary) font-mono'>[{it.storeType === 1 ? 'DMM' : 'DLsite'}] {it.storeId}</div>
            <Button text='削除' onclick={() => remove(it)} />
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
