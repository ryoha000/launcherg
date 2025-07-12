<script lang='ts'>
  import Work from '@/components/Work/Work.svelte'
  import { works } from '@/store/works'

  interface Props {
    params: { id: string }
  }

  const { params }: Props = $props()
  const workPromise = $derived(works.get(+params.id))
</script>

{#await workPromise}
  <div class='w-full h-full flex items-center justify-center'>
    <span class='text-gray-500'>Loading...</span>
  </div>
{:then work}
  <div class='w-full h-full'>
    <Work {work} />
  </div>
{:catch error}
  <div class='w-full h-full flex items-center justify-center'>
    <span class='text-red-500'>Error: {error.message}</span>
  </div>
{/await}
