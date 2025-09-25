<script lang='ts'>
  import type { SidebarWorkItem } from '@/store/sidebarCollectionElements'
  import { route } from '@mateothegreat/svelte5-router'
  import { convertFileSrc } from '@tauri-apps/api/core'

  interface Props { collectionElement: SidebarWorkItem }

  const { collectionElement }: Props = $props()

  const imgSrc = $derived(collectionElement.thumbnail?.path ? convertFileSrc(collectionElement.thumbnail.path) : '')
</script>

<div
  class='relative h-full w-full cursor-pointer transition-all hover:z-10 focus-within:scale-110 hover:scale-115 focus-within:shadow-md hover:shadow-md'
>
  <a
    tabIndex={0}
    href={`/works/${collectionElement.id}?gamename=${collectionElement.title}`}
    use:route
    class='block h-full w-full'
  >
    {#if collectionElement.thumbnail?.path}
      <img
        decoding='async'
        class='h-full w-full rounded object-contain'
        src={imgSrc}
        alt={`${collectionElement.title}のサムネイル`}
      />
    {:else}
      <div
        class='h-full w-full flex items-center justify-center border rounded bg-bg-primary px-6 text-(body text-primary) font-bold'
      >
        {collectionElement.title}
      </div>
    {/if}
  </a>
</div>
