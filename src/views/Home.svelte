<script lang="ts">
  import { commandGetCollectionElement } from "@/lib/command";
  import Icon from "/icon.png";
  import { link } from "svelte-spa-router";
  import LinkText from "@/components/UI/LinkText.svelte";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import ZappingGames from "@/components/Home/ZappingGames.svelte";

  const memoRegex = /^smde_memo-(\d+)$/;
  const memoPromises = Promise.all(
    Object.keys(localStorage)
      .map((v) => +(v.match(memoRegex)?.[1] ?? "0"))
      .filter((v) => v)
      .map((v) => commandGetCollectionElement(v))
  );

  let isOpenGettingStarted = true;
</script>

<div class="w-full h-full overflow-y-auto">
  <div class="p-8 space-y-8">
    <div class="flex items-center gap-2 w-full">
      <img src={Icon} alt="launcherg icon" class="h-12" />
      <div class="font-logo text-(8 text-primary)">Launcherg</div>
    </div>
    {#if $sidebarCollectionElements.length === 0 && isOpenGettingStarted}
      <div
        class="space-y-2 p-4 border-(border-primary solid ~) rounded max-w-120"
      >
        <div class="flex items-center">
          <div class="text-(text-primary h3) font-medium">Getting started</div>
        </div>
        <div class="text-(text-tertiary body)">
          持っているゲームをこのランチャーに登録してみましょう。左のサイドバーにある「Add」ボタンから自動で追加できます。
        </div>
      </div>
    {/if}
    <div class="space-y-2">
      <div class="text-(text-primary h3) font-medium">Help</div>
      <LinkText
        href="https://youtu.be/GCTj6eRRgAM?si=WRFuBgNErwTJsNnk"
        text="1分でわかる Launcherg"
      />
      <LinkText
        href="https://ryoha000.hatenablog.com/entry/2023/09/24/003605"
        text="よくある Q&A"
      />
    </div>
    <div class="space-y-2">
      <div class="text-(text-primary h3) font-medium">Memo</div>
      {#await memoPromises then elements}
        {#if elements.length === 0 && $sidebarCollectionElements.length !== 0}
          <div
            class="space-y-2 p-4 border-(border-primary solid ~) rounded max-w-120"
          >
            <div class="flex items-center">
              <div class="text-(text-primary h3) font-medium">メモ機能</div>
            </div>
            <div class="text-(text-tertiary body)">
              このアプリにはメモ機能があります。サイドバーからゲームを選択して「Memo」ボタンを押すことでそのゲームについてメモを取ることができます。
            </div>
          </div>
        {:else}
          <div class="gap-1 flex-(~ col)">
            {#each elements as element (element.id)}
              <a
                use:link
                href="/memos/{element.id}?gamename={element.gamename}"
                class="text-(text-link body2) hover:underline-(1px text-link)"
              >
                メモ - {element.gamename}
              </a>
            {/each}
          </div>
        {/if}
      {/await}
    </div>
    <ZappingGames />
  </div>
</div>
