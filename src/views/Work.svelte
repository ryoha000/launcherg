<script lang="ts">
  import { works } from "@/store/works";
  import Work from "@/components/Work/Work.svelte";

  export let params: { id: string };

  $: workPromise = works.get(+params.id);
</script>

{#await workPromise}
  <div class="w-full h-full flex items-center justify-center">
    <span class="text-gray-500">Loading...</span>
  </div>
{:then work}
  <div class="w-full h-full">
    <Work {work} />
  </div>
{:catch error}
  <div class="w-full h-full flex items-center justify-center">
    <span class="text-red-500">Error: {error.message}</span>
  </div>
{/await}
