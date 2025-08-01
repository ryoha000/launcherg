<script lang='ts'>
  import { open } from '@tauri-apps/plugin-shell'
  import { createEventDispatcher } from 'svelte'

  const dispatch = createEventDispatcher()

  interface Props {
    href: string
    text?: string
    intercept?: boolean
    children?: import('svelte').Snippet
  }

  const {
    href,
    text = '',
    intercept = false,
    children,
  }: Props = $props()

  const handleClick = () => {
    if (intercept) {
      dispatch('click', { href })
    }
    else {
      open(href)
    }
  }
</script>

<button
  onclick={handleClick}
  class='block whitespace-nowrap bg-transparent text-(body2 text-link) underline-none underline-text-link transition-all hover:underline'
>
  {#if children}{@render children()}{:else}
    {text}
  {/if}
</button>
