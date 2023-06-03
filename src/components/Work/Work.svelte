<script lang="ts">
  import type { Work } from "@/lib/types";

  export let work: Work;
</script>

<div class="text-text-primary">{JSON.stringify(work)}</div>
