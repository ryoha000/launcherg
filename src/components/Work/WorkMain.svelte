<script lang='ts'>
  import type { WorkDetailsVm } from '@/lib/command'
  import type { Work } from '@/lib/types'
  import LinkButton from '@/components/UI/LinkButton.svelte'
  import Table from '@/components/UI/Table.svelte'
  import Actions from '@/components/Work/Actions.svelte'
  import LinkToSidebar from '@/components/Work/LinkToSidebar.svelte'
  import { seiya } from '@/store/seiya'

  interface Props {
    workDetail: WorkDetailsVm
    workInformation: Work | undefined
  }

  const { workDetail, workInformation }: Props = $props()

  const seiyaUrlPromise = $derived(seiya.getUrl(workDetail.title))
  const summaryValue = $derived([
    {
      label: 'ブランド',
      value: workInformation?.brandName ?? '不明',
      component: LinkToSidebar,
    },
    { label: '発売日', value: workInformation?.sellday ?? '不明' },
    { label: '平均プレイ時間', value: `${workInformation?.statistics.playTime}` },
    { label: '中央値', value: `${workInformation?.statistics.median}` },
    { label: 'データ数', value: `${workInformation?.statistics.count}` },
  ])
</script>

<div class='max-w-full space-y-4'>
  <div class='text-(h1 text-primary) font-bold'>{workDetail.title}</div>
  {#await seiyaUrlPromise then seiyaUrl}
    <Actions {workDetail} id={workDetail.id} {seiyaUrl} />
  {/await}
  <div class='flex items-center'>
    <LinkButton href={workInformation?.officialHomePage ?? ''} text='Official' withIcon />
    {#if workInformation?.erogamescapeId}
      <LinkButton
        href='https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game={workInformation.erogamescapeId}'
        text='ErogameScape'
        withIcon
      />
    {:else}
      <span class='text-gray-500'>EGS ID未連携</span>
    {/if}
    {#await seiyaUrlPromise then url}
      <LinkButton href={url} text='誠也の部屋' withIcon />
    {/await}
  </div>
  <Table title='Summary' rows={summaryValue} />
</div>
