<script lang='ts'>
  import type { Work } from '@/lib/types'
  import LinkButton from '@/components/UI/LinkButton.svelte'
  import Table from '@/components/UI/Table.svelte'
  import Actions from '@/components/Work/Actions.svelte'
  import LinkToSidebar from '@/components/Work/LinkToSidebar.svelte'
  import { seiya } from '@/store/seiya'

  interface Props {
    work: Work
  }

  const { work }: Props = $props()

  const seiyaUrlPromise = $derived(seiya.getUrl(work.name))
  const summaryValue = $derived([
    {
      label: 'ブランド',
      value: work.brandName,
      component: LinkToSidebar,
    },
    { label: '発売日', value: work.sellday },
    { label: '平均プレイ時間', value: `${work.statistics.playTime}` },
    { label: '中央値', value: `${work.statistics.median}` },
    { label: 'データ数', value: `${work.statistics.count}` },
  ])
</script>

<div class='space-y-4 max-w-full'>
  <div class='text-(h1 text-primary) font-bold'>{work.name}</div>
  {#await seiyaUrlPromise then seiyaUrl}
    <Actions id={work.id} name={work.name} {seiyaUrl} />
  {/await}
  <div class='flex items-center'>
    <LinkButton href={work.officialHomePage} text='Official' withIcon />
    <LinkButton
      href='https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game={work.id}'
      text='ErogameScape'
      withIcon
    />
    {#await seiyaUrlPromise then url}
      <LinkButton href={url} text='誠也の部屋' withIcon />
    {/await}
  </div>
  <Table title='Summary' rows={summaryValue} />
</div>
