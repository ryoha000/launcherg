<script lang="ts">
  export let value: string;
  export let placeholder: string = "";

  let input: HTMLInputElement | null = null;
</script>

<svelte:window
  on:keydown={(e) => {
    if (e.key === "/") {
      const active = document.activeElement;
      if (
        active &&
        (active.tagName === "input" || active.tagName === "textarea")
      ) {
        return;
      }
      setTimeout(() => {
        if (input) {
          input.focus();
        }
      }, 20);
    }
  }}
/>
<div
  class="border-(2px solid transparent) focus-within:border-accent-accent rounded transition-all"
>
  <div
    class="group w-full flex items-center gap-2 px-2 border border-(border-primary solid) rounded bg-bg-secondary hover:bg-bg-primary focus-within:(border-transparent bg-bg-primary) transition-all"
  >
    <div class="w-4 h-4 i-material-symbols-search color-text-primary" />
    <input
      bind:this={input}
      bind:value
      {placeholder}
      class="w-full text-(body2 text-primary) bg-bg-secondary group-hover:bg-bg-primary focus:bg-bg-primary placeholder-text-placeholder transition-all"
    />
  </div>
</div>
