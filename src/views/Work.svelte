<script lang='ts'>
  import Work from '@/components/Work/Work.svelte'
  import { works } from '@/store/works'

  let { route }: { route?: { result: { path: { params: { id?: string } } } } } = $props()
  let id = $derived(route?.result?.path?.params?.id || '')
  let workPromise = $derived(works.get(+id))
</script>

{#await workPromise}
  <div class='h-full w-full flex items-center justify-center'>
    <span class='text-gray-500'>Loading...</span>
  </div>
{:then work}
  {#if work}
    <div class='h-full w-full'>
      <Work {work} />
    </div>
  {:else}
    <div class='h-full w-full flex items-center justify-center'>
      <span class='text-red-500'>作品が見つかりません</span>
    </div>
  {/if}
{:catch error}
  <div class='h-full w-full flex items-center justify-center'>
    <span class='text-red-500'>Error: {error.message}</span>
  </div>
{/await}
