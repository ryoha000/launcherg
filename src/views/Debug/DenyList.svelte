<script lang='ts'>
  // 統合後: workId のみを扱う
  type WorkOmitItemVm = number
  import { get } from 'svelte/store'
  import Button from '@/components/UI/Button.svelte'
  import { useAddWorkOmitMutation, useRemoveWorkOmitMutation, useWorkOmitQuery } from '@/lib/data/queries/workOmit'

  // 旧UIのストア種別は廃止

  const denyListQuery = useWorkOmitQuery()
  const addMutation = useAddWorkOmitMutation()
  const removeMutation = useRemoveWorkOmitMutation()
  let items = $state<WorkOmitItemVm[]>([])
  $effect(() => {
    const q = get(denyListQuery)
    items = q.data ?? []
  })
  let workIdInput = $state('')
  let loading = $derived.by(() => {
    const q = get(denyListQuery)
    const a = get(addMutation)
    const r = get(removeMutation)
    return !!(q.isLoading || a.isPending || r.isPending)
  })

  const add = async () => {
    const wid = Number(workIdInput.trim())
    if (!wid || Number.isNaN(wid))
      return
    await get(addMutation).mutateAsync({ workId: wid })
    workIdInput = ''
  }

  const remove = async (it: WorkOmitItemVm) => {
    await get(removeMutation).mutateAsync({ workId: it })
  }
</script>

<div class='mx-auto h-full max-w-3xl overflow-y-auto p-6'>
  <div class='mb-4 flex items-end gap-3'>
    <div class='flex-1'>
      <div class='mb-1 text-(sm text-secondary)'>Work ID</div>
      <input bind:value={workIdInput} class='w-full border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)' placeholder='works.id を入力' />
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
            <div class='text-(sm text-primary) font-mono'>workId: {it}</div>
            <Button text='削除' onclick={() => remove(it)} />
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
