<script lang='ts'>
  import { fly } from 'svelte/transition'

  interface Props {
    appendClass: string
    back?: boolean
    show: boolean
    onclick?: (e: Event) => void
  }

  const { appendClass, back = false, show, onclick }: Props = $props()

  const wrappedOnclick = (e: Event) => {
    e.stopPropagation()
    if (onclick) {
      onclick(e)
    }
  }
</script>

{#if show}
  <div
    class='absolute p-(y-2 l-6) {appendClass} bottom--1 top--1 flex items-center'
    style='background: linear-gradient(90deg, rgba(34,39,46,0) 0%, rgba(34,39,46,0.773546918767507) 15%, rgba(34,39,46,1) 30%, rgba(34,39,46,1) 100%);'
    class:rotate-180={back}
    transition:fly={{ x: 10, duration: 150 }}
  >
    <button
      class='rounded-full bg-transparent p-1 transition-all hover:bg-bg-button-hover'
      onclick={wrappedOnclick}
      aria-label='Scroll search attributes'
    >
      <div
        class='i-material-symbols-arrow-forward-ios-rounded h-4 w-4 color-text-primary'
      ></div>
    </button>
  </div>
{/if}
