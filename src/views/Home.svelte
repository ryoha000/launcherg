<script lang="ts">
  import { commandGetCollectionElement } from "@/lib/command";
  import Logo from "/logo.png";
  import { link } from "svelte-spa-router";
  import LinkText from "@/components/UI/LinkText.svelte";

  const memoRegex = /^smde_memo-(\d+)$/;
  const memoPromises = Promise.all(
    Object.keys(localStorage)
      .map((v) => +(v.match(memoRegex)?.[1] ?? "0"))
      .filter((v) => v)
      .map((v) => commandGetCollectionElement(v))
  );
</script>

<div class="p-8 space-y-8">
  <div>
    <img src={Logo} alt="launcherg logo" class="h-12" />
    <div class="text-(text-tertiary h3) font-bold">Hgame laucher</div>
  </div>
  <div class="space-y-2">
    <div class="text-(text-primary h3) font-medium">Help</div>
    <LinkText href="https://google.com" text="TODO: おきかえ" />
  </div>
  <div class="space-y-2">
    <div class="text-(text-primary h3) font-medium">Memo</div>
    {#await memoPromises then elements}
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
    {/await}
  </div>
</div>
