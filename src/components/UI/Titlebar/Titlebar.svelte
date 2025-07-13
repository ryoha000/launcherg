<script lang='ts'>
  import { onMount } from 'svelte'
  import ControlButton from '@/components/UI/Titlebar/ControlButton.svelte'
  import Close from '@/components/UI/Titlebar/Icons/Close.svelte'
  import Maximize from '@/components/UI/Titlebar/Icons/Maximize.svelte'
  import MaximizeRestore from '@/components/UI/Titlebar/Icons/MaximizeRestore.svelte'
  import Minimize from '@/components/UI/Titlebar/Icons/Minimize.svelte'
  import { useWindow } from '@/components/UI/Titlebar/window.svelte'

  const { isMaximized, isFocused, initialize, cleanup, minimize, close, toggleMaximize } = useWindow()
  onMount(() => {
    initialize()
    return () => {
      cleanup()
    }
  })
</script>

<div class='grid grid-cols-[1fr_auto] items-center h-8 bg-[#202020] text-(text-primary)'>
  <div data-tauri-drag-region class='h-full'></div>
  <div class='ml-auto flex-(~ cols) h-full'>
    <ControlButton variant='normal' isFocused={isFocused()} onclick={minimize}>
      <Minimize />
    </ControlButton>
    <ControlButton variant='normal' isFocused={isFocused()} onclick={toggleMaximize}>
      {#if isMaximized()}
        <MaximizeRestore />
      {:else}
        <Maximize />
      {/if}
    </ControlButton>
    <ControlButton variant='danger' isFocused={isFocused()} onclick={close}>
      <Close />
    </ControlButton>
  </div>
</div>
