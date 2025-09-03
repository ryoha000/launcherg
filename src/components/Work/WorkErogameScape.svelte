<script lang='ts'>
  import WorkLayout from '@/components/Work/WorkLayout.svelte'
  import { useWorkDetailsQuery } from '@/lib/data/queries/workDetails'
  import { works } from '@/store/works'

  interface Props {
    collectionElementId: number
  }

  const { collectionElementId }: Props = $props()

  const workDetailQuery = useWorkDetailsQuery(collectionElementId)
  const workInformationPromise = $derived.by(async () => {
    const erogameScapeId = $workDetailQuery.data?.erogamescapeId
    if (!erogameScapeId) {
      return undefined
    }
    return await works.get(erogameScapeId)
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
