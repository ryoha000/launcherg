<script lang="ts">
  import { location, querystring } from "svelte-spa-router";
  import type { Work as TWork } from "@/lib/types";
  import { works } from "@/store/works";
  import Work from "@/components/Work/Work.svelte";
  import { fade } from "svelte/transition";

  let work: TWork | null = null;
  export let params: { id: number };

  $: {
    (async () => {
      let _work = await works.get(params.id);
      if (work?.id !== _work.id) {
        work = _work;
      }
    })();
  }
</script>

{#if work}
  {#key work.id}
    <div class="w-full h-full" transition:fade={{ duration: 150 }}>
      <Work {work} />
    </div>
  {/key}
{/if}
