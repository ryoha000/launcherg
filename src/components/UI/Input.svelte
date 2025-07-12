<script lang='ts'>
  import { createEventDispatcher, onMount, tick } from 'svelte'

  interface Props {
    label?: string
    value: string
    placeholder?: string
    autofocus?: boolean
  }

  let {
    label = '',
    value = $bindable(),
    placeholder = '',
    autofocus = false,
  }: Props = $props()

  const dispatcher = createEventDispatcher<{ update: { value: string } }>()

  let input: HTMLInputElement | null = $state(null)

  onMount(async () => {
    if (!autofocus) {
      return
    }
    await tick()
    input?.focus()
  })
</script>

<label>
  {#if label}
    <div class='text-(text-primary body) font-medium mb-1'>{label}</div>
  {/if}
  <div
    class='w-full border-(2px solid transparent) focus-within:border-accent-accent rounded transition-all'
  >
    <input
      bind:this={input}
      bind:value
      type='text'
      oninput={e => dispatcher('update', { value: e.currentTarget.value })}
      {placeholder}
      class='w-full border border-(border-primary solid) rounded bg-bg-primary p-(x-3 y-1) text-(input text-primary) transition-all focus:border-transparent placeholder-ui-tertiary'
    />
  </div>
</label>
