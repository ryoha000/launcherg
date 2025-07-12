<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";

  interface Props {
    href: string;
    text: string;
    withIcon: boolean;
  }

  let { href, text, withIcon }: Props = $props();
</script>

<button
  onclick={() => open(href)}
  class="rounded flex h-8 items-center px-3 gap-2 bg-transparent transition-all hover:bg-bg-button-hover"
>
  <div class="whitespace-nowrap text-(body2 text-link) font-medium">{text}</div>
  {#if withIcon}
    <div
      class="w-4 h-4 color-text-link i-material-symbols-open-in-new-rounded"
></div>
  {/if}
</button>
