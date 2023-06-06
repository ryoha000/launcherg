<script lang="ts">
  import FullModal from "@/components/UI/FullModal.svelte";

  let isOpen = false;
</script>

<div>home</div>
<button on:click={() => (isOpen = true)}>hoge</button>
<FullModal bind:isOpen />
