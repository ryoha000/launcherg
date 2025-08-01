<script lang='ts'>
  import type { CollectionElement } from '@/lib/types'
  import { route } from '@mateothegreat/svelte5-router'
  import { convertFileSrc } from '@tauri-apps/api/core'

  interface Props {
    collectionElement: CollectionElement
  }

  const { collectionElement }: Props = $props()

  const iconSrc = $derived(convertFileSrc(collectionElement.icon))
</script>

<div
  class='flex items-center overflow-hidden rounded py-1 pl-2 transition-all hover:bg-bg-secondary'
>
  <a
    href={`/works/${collectionElement.id}?gamename=${collectionElement.gamename}`}
    class='h-12 w-full flex flex-(1) items-center gap-2 pr-2'
    use:route
  >
    <img
      alt='{collectionElement.gamename}_icon'
      src='{iconSrc}?{collectionElement.registeredAt}'
      class='h-10 w-10 rounded'
      loading='lazy'
    />
    <div class='max-h-full text-(body text-primary) font-bold'>
      {collectionElement.gamename}
    </div>
  </a>
</div>
