<script lang='ts'>
  import type { Props as TippyOption } from 'tippy.js'
  import ButtonBase from '@/components/UI/ButtonBase.svelte'

  interface Props {
    icon: string
    tooltip?: TippyOption | undefined
  }

  const { icon, tooltip = undefined }: Props = $props()
</script>

<ButtonBase appendClass='h-8 w-8 flex items-center justify-center' {tooltip}>
  <div class='color-ui-tertiary w-5 h-5 {icon}'></div>
</ButtonBase>
