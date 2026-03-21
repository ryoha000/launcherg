<script lang='ts'>
  interface Props {
    value: number
  }

  const { value }: Props = $props()

  const formatted = $derived.by(() => {
    const totalSeconds = Math.max(0, Math.floor(value))
    const hours = Math.floor(totalSeconds / 3600)
    const minutes = Math.floor((totalSeconds % 3600) / 60)
    const seconds = totalSeconds % 60

    const pad = (n: number) => String(n).padStart(2, '0')
    if (hours > 0) {
      return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`
    }
    return `${pad(minutes)}:${pad(seconds)}`
  })
</script>

<div class='flex items-center justify-end gap-2 text-(sm text-secondary) font-medium'>
  <div class='text-text-primary font-mono'>{formatted}</div>
</div>
