<script lang='ts'>
  import type { ScanTaskStatus } from '@/components/Sidebar/useImportProgress.svelte'

  interface Props {
    status: ScanTaskStatus
    title: string
    description: string
  }

  const { status, title, description }: Props = $props()

  const statusIconClass = $derived.by(() => {
    switch (status) {
      case 'idle':
        return 'i-material-symbols-radio-button-unchecked-rounded color-text-tertiary'
      case 'running':
        return 'i-material-symbols-sync-rounded animate-spin color-accent-accent'
      case 'done':
        return 'i-material-symbols-check-circle-rounded color-accent-success'
      default:
        throw new Error(`Unknown status: ${status satisfies never}`)
    }
  })

  const statusLabelClass = $derived.by(() => {
    switch (status) {
      case 'idle':
        return 'text-text-tertiary'
      case 'running':
        return 'text-accent-accent'
      case 'done':
        return 'font-medium text-accent-success'
      default:
        throw new Error(`Unknown status: ${status satisfies never}`)
    }
  })
</script>

<div class='py-1'>
  <div class='flex items-start gap-3'>
    <div class={`mt-0.5 h-5 w-5 shrink-0 ${statusIconClass}`}></div>
    <div class='min-w-0 flex-1'>
      <div class='flex items-center gap-2'>
        <div class='text-(body text-primary) font-semibold'>{title}</div>
        <div class={`text-[11px] leading-none ${statusLabelClass}`}>
          {#if status === 'idle'}
            Pending
          {:else if status === 'running'}
            Running
          {:else}
            Done
          {/if}
        </div>
      </div>
      <div
        class='mt-1 overflow-hidden text-(ellipsis body2 text-secondary) whitespace-nowrap'
        title={description}
      >
        {description}
      </div>
    </div>
  </div>
</div>
