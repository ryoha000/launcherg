<script lang='ts'>
  import type { WorkDetailsVm } from '@/lib/command'
  import type { Work } from '@/lib/types'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'
  import Detail from '@/components/Work/Detail.svelte'
  import WorkImage from '@/components/Work/WorkImage.svelte'
  import WorkMain from '@/components/Work/WorkMain.svelte'

  interface Props {
    workDetail: WorkDetailsVm
    workInformation: Work | undefined
  }

  const { workDetail, workInformation }: Props = $props()

  const imgUrl = $derived.by(() => {
    if (workInformation) {
      return workInformation.imgUrl
    }
    if (workDetail.thumbnail) {
      return convertFileSrc(workDetail.thumbnail)
    }
    return ''
  })

  let isLandscape = $state(false)
  onMount(() => {
    const image = new Image()
    image.addEventListener('load', () => {
      isLandscape = image.width > image.height
    })
    if (imgUrl) {
      image.src = imgUrl
    }
  })
</script>

{#if isLandscape}
  <div class='max-w-192 min-h-0 w-full p-(x-8 y-8) space-y-8'>
    <div class='w-full space-y-8'>
      <WorkImage name={workDetail.title} src={imgUrl} />
      <WorkMain {workDetail} {workInformation} />
    </div>
    {#if workInformation}
      <Detail work={workInformation} />
    {/if}
  </div>
{:else}
  <div class='max-w-256 min-h-0 w-full p-(x-8 y-8) space-y-8'>
    <div
      class='grid grid-cols-[repeat(auto-fill,_minmax(320px,_1fr))] w-full gap-8'
    >
      <WorkImage name={workDetail.title} src={imgUrl} />
      <WorkMain {workDetail} {workInformation} />
    </div>
    {#if workInformation}
      <Detail work={workInformation} />
    {/if}
  </div>
{/if}
