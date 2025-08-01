<script lang='ts'>
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
    onconfirm?: () => void
    oncancel?: () => void
    onclose?: () => void
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
    onconfirm,
    oncancel,
    onclose,
  }: Props = $props()
</script>

<ModalBase
  {isOpen}
  panelClass={maxWidth || 'max-w-160'}
  {fullmodal}
  {onclose}
>
  <div class='grid grid-rows-[min-content_1fr_min-content] h-full'>
    <div
      class='flex items-center border-(b-1px border-primary solid) rounded-t-lg bg-bg-secondary {headerClass}'
    >
      <div class='px-4 text-(body text-primary) font-medium'>
        {title}
      </div>
      <button
        onclick={onclose}
        class='ml-auto bg-transparent p-4 color-text-tertiary transition-all hover:color-text-primary'
        tabindex={autofocusCloseButton ? 0 : -1}
        aria-label='Close modal'
      >
        <div class='i-iconoir-cancel h-5 w-5'></div>
      </button>
    </div>
    <div class:p-4={withContentPadding} class='overflow-y-auto'>
      {@render children?.()}
    </div>
    {#if withFooter}
      {#if footer}{@render footer()}{:else}
        <div class='flex items-center border-(t-1px border-primary solid) p-4'>
          <div class='ml-auto flex items-center gap-2'>
            <Button text={cancelText} onclick={oncancel} />
            <Button
              variant='success'
              disabled={confirmDisabled}
              text={confirmText}
              onclick={onconfirm}
            />
          </div>
        </div>
      {/if}
    {/if}
  </div>
</ModalBase>
