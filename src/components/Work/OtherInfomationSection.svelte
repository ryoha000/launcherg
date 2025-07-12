<script lang='ts'>
  interface Props {
    label: string
    value: string | number | undefined | null
  }

  const { label, value }: Props = $props()
</script>

<div class='space-y-2'>
  <div class='text-(body2 text-primary) font-medium'>{label}</div>
  <div class='text-(body2 text-secondary)'>
    {value}
  </div>
</div>
