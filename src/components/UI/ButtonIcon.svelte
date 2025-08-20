<script lang='ts'>
  import type { Props as TippyOption } from 'tippy.js'
  import ButtonBase from '@/components/UI/ButtonBase.svelte'

  interface Props {
    icon: string
    tooltip?: TippyOption | undefined
    onclick?: (event: Event) => void
  }

  const { icon, tooltip = undefined, onclick }: Props = $props()
</script>

<ButtonBase appendClass='h-8 w-8 flex items-center justify-center' {tooltip} {onclick}>
  <div class='h-5 w-5 color-ui-tertiary {icon}'></div>
</ButtonBase>
