<script lang="ts">
  import { open } from "@tauri-apps/api/shell";

  export let href: string;
  export let text: string = "";
</script>

<button
  on:click={() => open(href)}
  class="text-(body2 text-link) block whitespace-nowrap underline-text-link hover:underline bg-transparent transition-all underline-none"
>
  <slot>
    {text}
  </slot>
</button>
