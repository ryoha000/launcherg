<script lang='ts'>
  import { open } from '@tauri-apps/plugin-shell'

  interface Props {
    href: string
    text: string
    withIcon: boolean
  }

  const { href, text, withIcon }: Props = $props()
</script>

<button
  onclick={() => open(href)}
  class='h-8 flex items-center gap-2 rounded bg-transparent px-3 transition-all hover:bg-bg-button-hover'
>
  <div class='whitespace-nowrap text-(body2 text-link) font-medium'>{text}</div>
  {#if withIcon}
    <div
      class='i-material-symbols-open-in-new-rounded h-4 w-4 color-text-link'
    ></div>
  {/if}
</button>
