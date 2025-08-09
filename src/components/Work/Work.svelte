<script lang='ts'>
  import type { Work } from '@/lib/types'
  import WorkErogameScape from '@/components/Work/WorkErogameScape.svelte'

  interface Props {
    work: Work
  }

  const { work }: Props = $props()
</script>

<WorkErogameScape {work} />
