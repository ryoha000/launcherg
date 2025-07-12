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
  class='border-(2px solid transparent) focus-within:border-accent-accent rounded transition-all'
>
  <div
    class='group w-full flex items-center gap-2 px-2 py-1 border border-(border-primary solid) rounded bg-bg-secondary hover:bg-bg-primary focus-within:(border-transparent bg-bg-primary) transition-all relative'
  >
    <div class='w-5 h-5 i-material-symbols-search color-text-primary'></div>
    <input
      bind:this={input}
      bind:value
      {placeholder}
      class='w-full text-(body2 text-primary) bg-bg-secondary group-hover:bg-bg-primary focus:bg-bg-primary placeholder-text-placeholder transition-all'
    />
    {#if value !== ''}
      <button
        onclick={() => (value = '')}
        class='absolute right-2 w-5 h-5 i-material-symbols-cancel-outline-rounded color-text-primary'
      ></button>
    {/if}
  </div>
</div>
