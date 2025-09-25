<script lang='ts'>
  import { onMount } from 'svelte'
  import { get } from 'svelte/store'
  import WorkLayout from '@/components/Work/WorkLayout.svelte'
  import { useWorkDetailsByWorkIdQuery } from '@/lib/data/queries/workDetails'
  import { useEvent } from '@/lib/event'
  import { works } from '@/store/works'

  interface Props { workId: number }

  const { workId }: Props = $props()

  const workDetailQuery = useWorkDetailsByWorkIdQuery(workId)
  const appEvent = useEvent()
  const workInformationPromise = $derived.by(async () => {
    const erogameScapeId = $workDetailQuery.data?.erogamescapeId
    if (!erogameScapeId) {
      return undefined
    }
    return await works.get(erogameScapeId)
  })

  onMount(() => {
    void appEvent.startListen('appSignal:refetchWork', ({ event }) => {
      if (event.type !== 'refetchWork')
        return

      const query = get(workDetailQuery)
      const current = query.data
      if (!current || current.id !== event.payload.workId)
        return

      void query.refetch()
    })

    return () => {
      appEvent.stopAll()
    }
  })
</script>

<div class='h-full w-full overflow-x-hidden overflow-y-auto'>
  <div class='min-h-0 w-full flex justify-center'>
    {#await workInformationPromise}
      <div class='h-full w-full flex items-center justify-center'>
        <span class='text-gray-500'>Loading...</span>
      </div>
    {:then workInformation}
      {#if $workDetailQuery.data}
        <WorkLayout workDetail={$workDetailQuery.data} {workInformation} />
      {:else}
        <div class='h-full w-full flex items-center justify-center'>
          <span class='text-gray-500'>Loading...</span>
        </div>
      {/if}
    {/await}
  </div>
</div>
