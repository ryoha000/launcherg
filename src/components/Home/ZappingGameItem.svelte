<script lang='ts'>
  import type { CollectionElement } from '@/lib/types'
  import { route } from '@mateothegreat/svelte5-router'
  import { convertFileSrc } from '@tauri-apps/api/core'

  interface Props {
    collectionElement: CollectionElement
  }

  const { collectionElement }: Props = $props()

  const imgSrc = $derived(convertFileSrc(collectionElement.thumbnail))
</script>

<div
  class='hover:scale-115 hover:shadow-md focus-within:scale-110 focus-within:shadow-md transition-all cursor-pointer w-full h-full relative hover:z-10'
>
  <a
    tabIndex={0}
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    use:route
    class='block w-full h-full'
  >
    {#if collectionElement.thumbnailWidth && collectionElement.thumbnailHeight}
      <img
        decoding='async'
        class='object-contain rounded w-full h-full'
        src={imgSrc}
        alt={`${collectionElement.gamename}のサムネイル`}
      />
    {:else}
      <div
        class='text-(body text-primary) font-bold px-6 rounded border bg-bg-primary w-full h-full flex items-center justify-center'
      >
        {collectionElement.gamename}
      </div>
    {/if}
  </a>
</div>
