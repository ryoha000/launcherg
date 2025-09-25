<script lang='ts'>
  import type { SidebarWorkItem } from '@/store/sidebarCollectionElements'
  import { route } from '@mateothegreat/svelte5-router'
  import { convertFileSrc } from '@tauri-apps/api/core'

  interface Props {
    collectionElement: SidebarWorkItem
  }

  const { collectionElement }: Props = $props()

  const iconSrc = $derived(collectionElement.thumbnail?.path ? convertFileSrc(collectionElement.thumbnail.path) : '')
</script>

<div
  class='flex items-center overflow-hidden rounded py-1 pl-2 transition-all hover:bg-bg-secondary'
>
  <a
    href={`/works/${collectionElement.id}?gamename=${collectionElement.title}`}
    class='h-12 w-full flex flex-(1) items-center gap-2 pr-2'
    use:route
  >
    <img
      alt='{collectionElement.title}_icon'
      src={iconSrc}
      class='h-10 w-10 rounded'
      loading='lazy'
    />
    <div class='max-h-full text-(body text-primary) font-bold'>
      {collectionElement.title}
    </div>
  </a>
</div>
