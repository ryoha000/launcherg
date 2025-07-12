<script lang='ts'>
  import type { Props as TippyOption } from 'tippy.js'

  import { run } from 'svelte/legacy'
  import ButtonBase from '@/components/UI/ButtonBase.svelte'

  interface Props {
    leftIcon?: string
    rightIcon?: string
    appendClass?: string
    text?: string
    type?: 'button' | 'submit' | undefined
    tooltip?: Partial<TippyOption> | undefined
    variant?: Variant
    disabled?: boolean
    wrappable?: boolean
    onclick?: (e: Event) => void
  }

  const {
    leftIcon = '',
    rightIcon = '',
    appendClass = '',
    text = '',
    type = undefined,
    tooltip = undefined,
    variant = 'normal',
    disabled = false,
    wrappable = false,
    onclick,
  }: Props = $props()

  const iconSizeClass = 'w-4 h-4'

  let iconVarinatClass = $state('color-ui-tertiary')
  run(() => {
    if (variant === 'success') {
      iconVarinatClass = 'color-text-white'
    }
  })
</script>

<ButtonBase
  appendClass={`${appendClass} ${
    wrappable ? '' : 'h-8'
  } px-3 gap-2 flex items-center`}
  {variant}
  {type}
  {tooltip}
  {disabled}
  {onclick}
>
  {#if leftIcon}
    <div class={`${iconVarinatClass} ${iconSizeClass} ${leftIcon}`}></div>
  {/if}
  {#if text}
    <div
      class={`text-body2 font-medium ${wrappable ? '' : 'whitespace-nowrap'}`}
    >
      {text}
    </div>
  {/if}
  {#if rightIcon}
    <div class={`${iconVarinatClass} ${iconSizeClass} ${rightIcon}`}></div>
  {/if}
</ButtonBase>
