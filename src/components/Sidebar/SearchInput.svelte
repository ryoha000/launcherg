<script lang='ts'>
  interface Props {
    value: string
    placeholder?: string
  }

  let { value = $bindable(), placeholder = '' }: Props = $props()

  let input: HTMLInputElement | null = $state(null)
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === '/') {
      const active = document.activeElement
      if (
        active
        && (active.tagName === 'input' || active.tagName === 'textarea')
      ) {
        return
      }
      setTimeout(() => {
        if (input) {
          input.focus()
        }
      }, 20)
    }
  }}
/>
<div
  class='border-(2px transparent solid) rounded transition-all focus-within:border-accent-accent'
>
  <div
    class='group relative w-full flex items-center gap-2 border border-(border-primary solid) rounded bg-bg-secondary px-2 py-1 transition-all focus-within:(border-transparent bg-bg-primary) hover:bg-bg-primary'
  >
    <div class='i-material-symbols-search h-5 w-5 color-text-primary'></div>
    <input
      bind:this={input}
      bind:value
      {placeholder}
      class='placeholder-text-placeholder w-full bg-bg-secondary text-(body2 text-primary) transition-all focus:bg-bg-primary group-hover:bg-bg-primary'
    />
    {#if value !== ''}
      <button
        onclick={() => (value = '')}
        class='i-material-symbols-cancel-outline-rounded absolute right-2 h-5 w-5 color-text-primary'
        aria-label='Clear search input'
      ></button>
    {/if}
  </div>
</div>
