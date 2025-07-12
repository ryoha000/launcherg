<script lang="ts">
  import Detail from "@/components/Work/Detail.svelte";
  import WorkImage from "@/components/Work/WorkImage.svelte";
  import WorkLayout from "@/components/Work/WorkLayout.svelte";
  import WorkMain from "@/components/Work/WorkMain.svelte";
  import type { Work } from "@/lib/types";

  interface Props {
    work: Work;
  }

  let { work }: Props = $props();
</script>

<div class="h-full w-full overflow-x-hidden overflow-y-auto">
  <div class="w-full min-h-0 flex justify-center">
    {#key work.imgUrl}
      <WorkLayout {work} />
    {/key}
  </div>
</div>
