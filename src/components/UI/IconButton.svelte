<script lang='ts'>
  import ButtonBase from '@/components/UI/ButtonBase.svelte'

  interface Props {
    icon: string
  }

  const { icon }: Props = $props()
</script>

<ButtonBase appendClass='h-8 w-8 p-2' on:click>
  <div class={`color-ui-tertiary w-4 h-4 ${icon}`}></div>
</ButtonBase>
