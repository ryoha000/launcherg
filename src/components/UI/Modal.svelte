<script lang='ts'>
  import { createEventDispatcher } from 'svelte'
  import Button from '@/components/UI/Button.svelte'
  import ModalBase from '@/components/UI/ModalBase.svelte'

  interface Props {
    isOpen?: boolean
    autofocusCloseButton?: boolean
    maxWidth?: string
    headerClass?: string
    title?: string
    confirmText?: string
    cancelText?: string
    withFooter?: boolean
    withContentPadding?: boolean
    fullmodal?: boolean
    confirmDisabled?: boolean
    children?: import('svelte').Snippet
    footer?: import('svelte').Snippet
  }

  const {
    isOpen = false,
    autofocusCloseButton = false,
    maxWidth = '',
    headerClass = '',
    title = '',
    confirmText = '',
    cancelText = 'Cancel',
    withFooter = true,
    withContentPadding = true,
    fullmodal = false,
    confirmDisabled = false,
    children,
    footer,
  }: Props = $props()

  const dispatcher = createEventDispatcher<{
    confirm: {}
    cancel: {}
    close: {}
  }>()
</script>

<ModalBase
  {isOpen}
  panelClass={maxWidth || 'max-w-160'}
  {fullmodal}
  on:close
>
  <div class='grid grid-rows-[min-content_1fr_min-content] h-full'>
    <div
      class='flex items-center bg-bg-secondary border-(b-1px solid border-primary) rounded-t-lg {headerClass}'
    >
      <div class='px-4 text-(text-primary body) font-medium'>
        {title}
      </div>
      <button
        onclick={() => dispatcher('close')}
        class='ml-auto p-4 bg-transparent color-text-tertiary hover:color-text-primary transition-all'
        tabindex={autofocusCloseButton ? 0 : -1}
      >
        <div class='w-5 h-5 i-iconoir-cancel'></div>
      </button>
    </div>
    <div class:p-4={withContentPadding} class='overflow-y-auto'>
      {@render children?.()}
    </div>
    {#if withFooter}
      {#if footer}{@render footer()}{:else}
        <div class='flex items-center p-4 border-(t-1px solid border-primary)'>
          <div class='flex items-center ml-auto gap-2'>
            <Button text={cancelText} on:click={() => dispatcher('cancel')} />
            <Button
              variant='success'
              disabled={confirmDisabled}
              text={confirmText}
              on:click={() => dispatcher('confirm')}
            />
          </div>
        </div>
      {/if}
    {/if}
  </div>
</ModalBase>
