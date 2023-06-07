<script lang="ts">
  import { works } from "@/store/works";
  import Work from "@/components/Work/Work.svelte";

  export let params: { id: string };

  $: workPromise = works.get(+params.id);
</script>

{#await workPromise then work}
  <div class="w-full h-full">
    <Work {work} />
  </div>
{/await}
