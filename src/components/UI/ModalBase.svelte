<script lang='ts'>
  import { createDialog } from 'svelte-headlessui'
  import { fade, scale } from 'svelte/transition'
  import { portal } from '@/components/UI/portal'

  interface Props {
    isOpen?: boolean
    panelClass?: string
    fullmodal?: boolean
    children?: import('svelte').Snippet<[]>
    onclose?: () => void
  }

  const {
    isOpen = true,
    panelClass = '',
    fullmodal = false,
    children,
    onclose,
  }: Props = $props()

  const dialog = createDialog({ opened: isOpen, expanded: isOpen })

  $effect(() => {
    if (isOpen) {
      dialog.open()
    }
    else {
      dialog.close()
    }
  })

  const handleClose = (e: Event) => {
    e.preventDefault()
    onclose?.()
  }
</script>

{#if isOpen}
  <div class='fixed inset-0 z-10 w-full h-full' onclose={handleClose} use:portal>
    <div class='relative p-12 w-full h-full'>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div transition:fade={{ duration: 100 }} class='absolute inset-0 z-20 bg-(bg-backdrop opacity-80)' onclick={handleClose}></div>
      <div
        transition:scale={{ delay: 100, duration: 200 }}
        class='relative w-full h-full z-30 m-auto {panelClass} overflow-hidden'
        class:h-full={fullmodal}
        use:dialog.modal
      >
        <div
          class='w-full h-full border-(~ solid border-primary) rounded-lg bg-bg-primary shadow min-h-0 max-h-full'
        >
          {@render children?.()}
        </div>
      </div>
    </div>
  </div>
{/if}
