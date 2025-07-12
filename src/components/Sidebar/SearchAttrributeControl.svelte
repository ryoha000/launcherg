<script lang="ts">
  import { createBubbler, stopPropagation } from 'svelte/legacy';

  const bubble = createBubbler();
  import { fly } from "svelte/transition";

  interface Props {
    appendClass: string;
    back?: boolean;
    show: boolean;
  }

  let { appendClass, back = false, show }: Props = $props();
</script>

{#if show}
  <div
    class="absolute p-(l-6 y-2) {appendClass} top--1 bottom--1 flex items-center"
    style="background: linear-gradient(90deg, rgba(34,39,46,0) 0%, rgba(34,39,46,0.773546918767507) 15%, rgba(34,39,46,1) 30%, rgba(34,39,46,1) 100%);"
    class:rotate-180={back}
    transition:fly={{ x: 10, duration: 150 }}
  >
    <button
      class="bg-transparent transition-all hover:bg-bg-button-hover rounded-full p-1"
      onclick={stopPropagation(bubble('click'))}
    >
      <div
        class="i-material-symbols-arrow-forward-ios-rounded w-4 h-4 color-text-primary"
></div>
    </button>
  </div>
{/if}
