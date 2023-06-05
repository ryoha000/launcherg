<script lang="ts">
  import LinkButton from "@/components/UI/LinkButton.svelte";
  import Table from "@/components/UI/Table.svelte";
  import Actions from "@/components/Work/Actions.svelte";
  import type { Work } from "@/lib/types";
  import { seiya } from "@/store/seiya";

  export let work: Work;

  $: seiyaUrlPromise = seiya.getUrl(work.name);
  $: summaryValue = [
    { label: "ブランド", value: work.brandName },
    { label: "発売日", value: work.sellday },
    { label: "中央値", value: `${work.statistics.median}` },
    { label: "平均値", value: `${work.statistics.average}` },
    { label: "データ数", value: `${work.statistics.count}` },
  ];
</script>

<div class="space-y-4 max-w-full">
  <Actions {...work} />
  <div class="flex items-center">
    <LinkButton href={work.officialHomePage} text="Official" withIcon />
    <LinkButton
      href="https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game={work.id}"
      text="ErogameScape"
      withIcon
    />
    {#await seiyaUrlPromise then url}
      <LinkButton href={url} text="誠也の部屋" withIcon />
    {/await}
  </div>
  <Table title="Summary" rows={summaryValue} />
</div>
