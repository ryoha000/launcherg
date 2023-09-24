<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let value: boolean;

  const dispather = createEventDispatcher<{ update: { value: boolean } }>();
  $: {
    dispather("update", { value });
  }
</script>

<input
  type="checkbox"
  checked={value}
  on:change={(e) => (value = e.currentTarget.checked)}
  class="hidden"
/>
{#if value}
  <div
    class="i-material-symbols-check-box-rounded color-border-button w-6 h-6"
  />
{:else}
  <div
    class="i-material-symbols-check-box-outline-blank color-border-button w-6 h-6"
  />
{/if}
