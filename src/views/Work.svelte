<script lang="ts">
  import { location, querystring } from "svelte-spa-router";
  import type { Work as TWork } from "@/lib/types";
  import { works } from "@/store/works";
  import Work from "@/components/Work/Work.svelte";

  let work: TWork | null = null;
  export let params: { id: number };

  $: {
    (async () => {
      work = await works.get(params.id);
    })();
  }
</script>

{#if work}
  <div class="w-full h-full">
    <Work {work} />
  </div>
{/if}
