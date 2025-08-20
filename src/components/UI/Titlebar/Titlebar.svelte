<script lang='ts'>
  import { goto } from '@mateothegreat/svelte5-router'
  import { onMount } from 'svelte'
  import ControlButton from '@/components/UI/Titlebar/ControlButton.svelte'
  import Close from '@/components/UI/Titlebar/Icons/Close.svelte'
  import Maximize from '@/components/UI/Titlebar/Icons/Maximize.svelte'
  import MaximizeRestore from '@/components/UI/Titlebar/Icons/MaximizeRestore.svelte'
  import Minimize from '@/components/UI/Titlebar/Icons/Minimize.svelte'
  import { useWindow } from '@/components/UI/Titlebar/window.svelte'

  const { isMaximized, isFocused, initialize, cleanup, minimize, close, toggleMaximize } = useWindow()

  function navigateToSettings() {
    goto('/settings')
  }

  onMount(() => {
    initialize()
    return () => {
      cleanup()
    }
  })
</script>

<div class='grid grid-cols-[1fr_auto] h-8 items-center bg-[#202020] text-(text-primary)'>
  <div data-tauri-drag-region class='h-full'></div>
  <div class='ml-auto h-full flex items-center'>
    <button class='h-7 w-7 flex items-center justify-center rounded-1 bg-transparent transition-all' aria-label='Open Settings' onclick={navigateToSettings}>
      <div class="h-5 w-5 {isFocused() ? 'color-[#e5e5e5' : 'color-[#797979]'} i-material-symbols-settings-outline-rounded"></div>
    </button>
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
