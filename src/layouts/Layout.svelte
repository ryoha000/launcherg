<script lang="ts">
  import Sidebar from "@/components/Sidebar/Sidebar.svelte";
</script>

<div class="grid grid-cols-[300px_1fr]">
  <Sidebar />
  <slot />
</div>
