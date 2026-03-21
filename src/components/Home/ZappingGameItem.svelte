<script lang='ts'>
  import type { SidebarWorkItem } from '@/store/sidebarWorks'
  import { route } from '@mateothegreat/svelte5-router'
  import { convertFileSrc } from '@tauri-apps/api/core'

  interface Props { work: SidebarWorkItem }

  const { work }: Props = $props()

  const imgSrc = $derived(work.thumbnail?.path ? convertFileSrc(work.thumbnail.path) : '')
  const hasSizedThumbnail = $derived(
    !!work.thumbnail?.path && !!work.thumbnail?.width && !!work.thumbnail?.height,
  )
</script>

<div
  class='relative h-full w-full cursor-pointer transition-all hover:z-10 focus-within:scale-110 hover:scale-115 focus-within:shadow-md hover:shadow-md'
>
  <a
    tabIndex={0}
    href={`/works/${work.id}?gamename=${work.title}`}
    use:route
    class='block h-full w-full'
  >
    {#if hasSizedThumbnail}
      <img
        decoding='async'
        class='h-full w-full rounded object-contain'
        src={imgSrc}
        alt={`${work.title}のサムネイル`}
      />
    {:else}
      <div
        class='h-full w-full flex items-center justify-center border rounded bg-bg-primary px-6 text-(body text-primary) font-bold'
      >
        {work.title}
      </div>
    {/if}
  </a>
</div>
