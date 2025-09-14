<script lang='ts'>
  interface Props {
    label: string
    value?: string | number | undefined | null
    children?: import('svelte').Snippet<[]>
  }

  const { label, value, children }: Props = $props()
</script>

<div class='space-y-1'>
  <div class='text-(body2 text-primary) font-medium'>{label}</div>
  <div class='text-(body2 text-secondary)'>
    {#if children}
      {@render children()}
    {:else}
      {value}
    {/if}
  </div>
</div>
