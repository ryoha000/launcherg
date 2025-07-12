<script lang='ts'>
  interface Props {
    noBorder?: boolean
    label: string
    children?: import('svelte').Snippet
  }

  const { noBorder = false, label, children }: Props = $props()
</script>

<div
  class="p-4 border-(solid border-primary) whitespace-nowrap text-(body2 text-primary) {noBorder
    ? 'border-0px'
    : 'border-t-1px'}"
>
  {label}
</div>
<div
  class="p-4 border-(solid border-primary) text-(body2 text-link) {noBorder
    ? 'border-0px'
    : 'border-t-1px'}"
>
  {@render children?.()}
</div>
