<script lang='ts'>
  interface Props {
    noBorder?: boolean
    label: string
    children?: import('svelte').Snippet
  }

  const { noBorder = false, label, children }: Props = $props()
</script>

<div
  class="whitespace-nowrap border-(border-primary solid) p-4 text-(body2 text-primary) {noBorder
    ? 'border-0px'
    : 'border-t-1px'}"
>
  {label}
</div>
<div
  class="border-(border-primary solid) p-4 text-(body2 text-link) {noBorder
    ? 'border-0px'
    : 'border-t-1px'}"
>
  {@render children?.()}
</div>
