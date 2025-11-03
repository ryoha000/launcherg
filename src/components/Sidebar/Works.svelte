<script lang='ts'>
  import type { SidebarWorkItemsWithLabel } from '@/store/sidebarWorks'
  import SimpleBar from 'simplebar'
  import WorkItem from '@/components/Sidebar/WorkItem.svelte'
  import ADisclosure from '@/components/UI/ADisclosure.svelte'

  interface Props { works: SidebarWorkItemsWithLabel[] }

  const { works }: Props = $props()

  const simplebar = (node: HTMLElement) => {
    const sb = new SimpleBar(node, { scrollbarMinSize: 64 })
    return {
      destroy() {
        sb.unMount()
      },
      update() {
        sb.recalculate()
      },
    }
  }
</script>

<div class='grid grid-(rows-[1fr]) h-full overflow-y-hidden'>
  {#if works.length}
    <div class='mt-2 min-h-0 flex-1'>
      <div use:simplebar class='h-full overflow-y-auto'>
        <div class='w-full'>
          {#each works as { label, elements } (label)}
            <ADisclosure {label} defaultOpen={works.length === 1}>
              {#each elements as work (work.id)}
                <WorkItem work={work} />
              {/each}
            </ADisclosure>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>
