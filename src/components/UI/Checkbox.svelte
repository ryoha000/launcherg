<script lang='ts'>
  import { createEventDispatcher } from 'svelte'

  import { run } from 'svelte/legacy'

  interface Props {
    value: boolean
  }

  let { value = $bindable() }: Props = $props()

  const dispather = createEventDispatcher<{ update: { value: boolean } }>()
  run(() => {
    dispather('update', { value })
  })
</script>

<input
  type='checkbox'
  checked={value}
  onchange={e => (value = e.currentTarget.checked)}
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
