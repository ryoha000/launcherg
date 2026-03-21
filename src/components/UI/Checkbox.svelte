<script lang='ts'>
  import { createEventDispatcher } from 'svelte'

  interface Props {
    value: boolean
    disabled?: boolean
  }

  let { value = $bindable(), disabled = $bindable() }: Props = $props()

  const dispather = createEventDispatcher<{ update: { value: boolean } }>()
</script>

<input
  type='checkbox'
  checked={value}
  disabled={disabled}
  onchange={(e) => {
    value = e.currentTarget.checked
    dispather('update', { value })
  }}
  class='hidden'
/>
{#if value}
  <div
    class='i-material-symbols-check-box-rounded h-6 w-6 color-border-button'
  ></div>
{:else}
  <div
    class='i-material-symbols-check-box-outline-blank h-6 w-6 color-border-button'
  ></div>
{/if}
