<script lang='ts'>
  import Sidebar from '@/components/Sidebar/Sidebar.svelte'
  import ATabList from '@/components/Tab/ATabList.svelte'

  interface Props {
    children?: import('svelte').Snippet
  }

  const { children }: Props = $props()
</script>

<div class='grid grid-cols-[min-content_1fr] overflow-hidden w-full h-full'>
  <Sidebar />
  <div
    class='h-full min-w-0 min-h-0 grid-(~ rows-[min-content_1fr]) overflow-hidden'
  >
    <ATabList />
    <div class='h-full min-h-0 min-w-0 w-full'>
      {@render children?.()}
    </div>
  </div>
</div>
