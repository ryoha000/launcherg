<script lang="ts">
  import { fly } from "svelte/transition";

  export let appendClass: string;
  export let back: boolean = false;
  export let show: boolean;
</script>

{#if show}
  <div
    class="absolute p-(l-4 y-2) {appendClass} top--1 bottom--1 flex items-center"
    style="background: linear-gradient(90deg, rgba(34,39,46,0) 0%, rgba(34,39,46,0.773546918767507) 15%, rgba(34,39,46,1) 40%, rgba(34,39,46,1) 100%);"
    class:rotate-180={back}
    transition:fly={{ x: 10, duration: 150 }}
  >
    <button
      class="bg-transparent transition-all hover:bg-bg-button-hover rounded-full p-1"
      on:click|stopPropagation
    >
      <div
        class="i-material-symbols-arrow-forward-ios-rounded w-4 h-4 color-text-primary"
      />
    </button>
  </div>
{/if}
