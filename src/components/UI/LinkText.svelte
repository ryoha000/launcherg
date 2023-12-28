<script lang="ts">
  import { open } from "@tauri-apps/api/shell";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let href: string;
  export let text: string = "";

  export let intercept: boolean = false;

  const handleClick = () => {
    if (intercept) {
      dispatch("click", { href });
    } else {
      open(href);
    }
  };
</script>

<button
  on:click={handleClick}
  class="text-(body2 text-link) block whitespace-nowrap underline-text-link hover:underline bg-transparent transition-all underline-none"
>
  <slot>
    {text}
  </slot>
</button>
