<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();


  interface Props {
    href: string;
    text?: string;
    intercept?: boolean;
    children?: import('svelte').Snippet;
  }

  let {
    href,
    text = "",
    intercept = false,
    children
  }: Props = $props();

  const handleClick = () => {
    if (intercept) {
      dispatch("click", { href });
    } else {
      open(href);
    }
  };
</script>

<button
  onclick={handleClick}
  class="text-(body2 text-link) block whitespace-nowrap underline-text-link hover:underline bg-transparent transition-all underline-none"
>
  {#if children}{@render children()}{:else}
    {text}
  {/if}
</button>
