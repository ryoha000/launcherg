<script lang="ts">
  import { onMount, tick } from "svelte";

  export let value: string;
  export let placeholder: string = "";
  export let autofocus = false;

  let input: HTMLInputElement | null = null;

  onMount(async () => {
    if (!autofocus) {
      return;
    }
    await tick();
    input?.focus();
  });
</script>

<div
  class="w-full border-(2px solid transparent) focus-within:border-accent-accent rounded transition-all"
>
  <input
    bind:this={input}
    bind:value
    {placeholder}
    class="w-full border border-(border-primary solid) rounded bg-bg-primary p-(x-3 y-1.5) text-(body2 text-primary) transition-all focus:border-transparent placeholder-ui-tertiary"
  />
</div>
