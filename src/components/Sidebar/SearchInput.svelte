<script lang="ts">
  export let value: string;
  export let placeholder: string = "";
</script>

<div
  class="border-(2px solid transparent) focus-within:border-accent-accent rounded transition-all"
>
  <div
    class="w-full flex items-center gap-2 px-2 border border-(border-primary solid) rounded bg-bg-primary focus:border-transparent transition-all"
  >
    <div class="w-4 h-4 i-material-symbols-search color-text-primary" />
    <input
      bind:value
      {placeholder}
      class="w-full text-(body2 text-primary) bg-bg-primary placeholder-text-placeholder"
    />
  </div>
</div>
