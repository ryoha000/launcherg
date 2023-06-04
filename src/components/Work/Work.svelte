<script lang="ts">
  import Detail from "@/components/Work/Detail.svelte";
  import WorkImage from "@/components/Work/WorkImage.svelte";
  import WorkMain from "@/components/Work/WorkMain.svelte";
  import type { Work } from "@/lib/types";

  export let work: Work;
</script>

<div class="h-full w-full overflow-x-hidden overflow-y-auto">
  <div class="w-full min-h-0 flex justify-center">
    <div class="p-(x-8 y-4) w-full min-h-0 max-w-256 space-y-8">
      <div class="text-(h1 text-primary) font-bold">{work.name}</div>
      <div
        class="grid grid-cols-[repeat(auto-fill,_minmax(320px,_1fr))] w-full gap-8"
      >
        <WorkImage name={work.name} src={work.imgUrl} />
        <WorkMain {work} />
      </div>
      <Detail {work} />
    </div>
  </div>
</div>
