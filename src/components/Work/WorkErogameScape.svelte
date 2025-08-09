<script lang='ts'>
  import type { Work } from '@/lib/types'
  import WorkLayout from '@/components/Work/WorkLayout.svelte'

  interface Props {
    work: Work
  }

  const { work }: Props = $props()
</script>

<div class='h-full w-full overflow-x-hidden overflow-y-auto'>
  <div class='min-h-0 w-full flex justify-center'>
    {#key work.imgUrl}
      <WorkLayout {work} />
    {/key}
  </div>
  
</div>

