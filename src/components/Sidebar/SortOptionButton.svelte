<script lang="ts">
  export let selected: boolean;
  export let text: string;
  export let withBorderTop = true;
</script>

<button
  on:click
  class="p-(x-4 y-2) flex items-center bg-transparent transition-all hover:(bg-bg-button-hover) {withBorderTop &&
    'border-(t-1px solid border-primary)'}"
>
  <div
    class="h-5 w-5 color-text-primary"
    class:i-material-symbols-check-small-rounded={selected}
  />
  <div class="text-(text-primary body2) font-bold whitespace-nowrap">
    {text}
  </div>
</button>
