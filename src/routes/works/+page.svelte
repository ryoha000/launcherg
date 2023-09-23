<script lang="ts">
  import { works } from "@/store/works";
  import Work from "@/components/Work/Work.svelte";
  import { page } from "$app/stores";

  $: workPromise = works.get(+$page.url.searchParams.get("id")!);
</script>

{#await workPromise then work}
  <div class="w-full h-full">
    <Work {work} />
  </div>
{/await}
