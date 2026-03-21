<script lang='ts'>
  import type { ScanProgressState } from '@/components/Sidebar/useImportProgress.svelte'
  import ElapsedTimeLabel from '@/components/Sidebar/ElapsedTimeLabel.svelte'
  import ScanTaskItem from '@/components/Sidebar/ScanTaskItem.svelte'
  import ModalBase from '@/components/UI/ModalBase.svelte'

  interface Props {
    isOpen: boolean
    progress: ScanProgressState
    panelClass?: string
    onclose?: () => void
  }

  const {
    isOpen,
    progress,
    panelClass = 'max-w-120',
    onclose,
  }: Props = $props()

  const formatNumber = (value: number) => new Intl.NumberFormat('ja-JP').format(value)

  const exploreDescription = $derived.by(() => {
    if (progress.explore.status === 'idle') {
      return 'Waiting to start'
    }
    if (progress.explore.status === 'done') {
      return `${formatNumber(progress.explore.discoveredCandidates)} candidates found`
    }
    return `${formatNumber(progress.explore.discoveredCandidates)} candidates found / Current: ${progress.explore.currentPath ?? '-'}`
  })

  const judgeDescription = $derived.by(() => {
    const judged = formatNumber(progress.judge.judgedCount)
    const recognized = formatNumber(progress.judge.recognizedCount)
    if (progress.judge.totalCandidates !== null) {
      return `${judged} / ${formatNumber(progress.judge.totalCandidates)} matched / ${recognized} recognized`
    }
    return `${judged} matched / ${recognized} recognized`
  })

  const imageDescription = $derived.by(() => {
    if (progress.images.status === 'idle') {
      return 'Waiting to start'
    }
    if (progress.images.status === 'done') {
      return `${formatNumber(progress.images.processedCount)} fetched`
    }
    if (progress.images.totalCount !== null) {
      return `${formatNumber(progress.images.processedCount)} / ${formatNumber(progress.images.totalCount)} fetched`
    }
    return `${formatNumber(progress.images.processedCount)} fetched`
  })
</script>

<ModalBase {isOpen} panelClass={panelClass} {onclose}>
  <div class='w-full p-5 space-y-4'>
    <div class='space-y-3'>
      <ScanTaskItem
        status={progress.explore.status}
        title='Scanning'
        description={exploreDescription}
      />
      <ScanTaskItem
        status={progress.judge.status}
        title='Matching'
        description={judgeDescription}
      />
      <ScanTaskItem
        status={progress.images.status}
        title='Fetching'
        description={imageDescription}
      />
    </div>
    <ElapsedTimeLabel value={progress.elapsedSeconds} />
  </div>
</ModalBase>
