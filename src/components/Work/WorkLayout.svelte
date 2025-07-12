<script lang='ts'>
  import type { Work } from '@/lib/types'
  import { onMount } from 'svelte'
  import Detail from '@/components/Work/Detail.svelte'
  import WorkImage from '@/components/Work/WorkImage.svelte'
  import WorkMain from '@/components/Work/WorkMain.svelte'

  interface Props {
    work: Work
  }

  const { work }: Props = $props()
  let isLandscape = $state(false)
  onMount(() => {
    const image = new Image()
    image.addEventListener('load', () => {
      isLandscape = image.width > image.height
    })
    image.src = work.imgUrl
  })
</script>

{#if isLandscape}
  <div class='p-(x-8 y-8) w-full min-h-0 max-w-192 space-y-8'>
    <div class='w-full space-y-8'>
      <WorkImage name={work.name} src={work.imgUrl} />
      <WorkMain {work} />
    </div>
    <Detail {work} />
  </div>
{:else}
  <div class='p-(x-8 y-8) w-full min-h-0 max-w-256 space-y-8'>
    <div
      class='grid grid-cols-[repeat(auto-fill,_minmax(320px,_1fr))] w-full gap-8'
    >
      <WorkImage name={work.name} src={work.imgUrl} />
      <WorkMain {work} />
    </div>
    <Detail {work} />
  </div>
{/if}
