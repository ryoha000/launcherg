<script lang='ts'>
  import type { CollectionElementsWithLabel } from '@/lib/types'
  import SimpleBar from 'simplebar'
  import CollectionElement from '@/components/Sidebar/CollectionElement.svelte'
  import ADisclosure from '@/components/UI/ADisclosure.svelte'

  interface Props {
    collectionElement: CollectionElementsWithLabel[]
  }

  const { collectionElement }: Props = $props()

  const simplebar = (node: HTMLElement) => {
    new SimpleBar(node, { scrollbarMinSize: 64 })
  }
</script>

<div class='grid-(~ rows-[1fr]) h-full overflow-y-hidden'>
  {#if collectionElement.length}
    <div class='flex-1 mt-2 min-h-0'>
      <div use:simplebar class='h-full overflow-y-auto'>
        <div class='w-full'>
          {#each collectionElement as { label, elements } (label)}
            <ADisclosure {label} defaultOpen={collectionElement.length === 1}>
              {#each elements as ele (ele.id)}
                <CollectionElement collectionElement={ele} />
              {/each}
            </ADisclosure>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>
