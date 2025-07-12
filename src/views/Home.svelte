<script lang='ts'>
  import { route } from '@mateothegreat/svelte5-router'
  import { derived } from 'svelte/store'
  import Button from '@/components/UI/Button.svelte'
  import LinkText from '@/components/UI/LinkText.svelte'
  import VirtualScroller from '@/components/UI/VirtualScroller.svelte'
  import VirtualScrollerMasonry from '@/components/UI/VirtualScrollerMasonry.svelte'
  import {
    commandGetCollectionElement,
    commandUpdateAllGameCache,
    commandUpdateCollectionElementThumbnails,
  } from '@/lib/command'
  import { scrapeAllGameCacheOnes } from '@/lib/scrape/scrapeAllGame'
  import { showErrorToast, showInfoToast } from '@/lib/toast'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'
  import Icon from '/icon.png'

  const memoRegex = /^smde_memo-(\d+)$/
  const memoPromises = Promise.all(
    Object.keys(localStorage)
      .map(v => +(v.match(memoRegex)?.[1] ?? '0'))
      .filter(v => v)
      .map(v => commandGetCollectionElement(v)),
  )

  const isOpenGettingStarted = true

  const shown = sidebarCollectionElements.shown
  const flattenShown = derived(shown, $shown =>
    $shown.flatMap(v => v.elements))

  let disabledRefetchThumbnail = $state(false)
  const refetchThumbnail = async () => {
    try {
      disabledRefetchThumbnail = true
      const ids = $flattenShown
        .filter(v => !v.thumbnailWidth && !v.thumbnailHeight)
        .map(v => v.id)
      const caches = await scrapeAllGameCacheOnes(ids)
      await commandUpdateAllGameCache(caches)
      await commandUpdateCollectionElementThumbnails(ids)
      await sidebarCollectionElements.refetch()
      showInfoToast('サムネイルの再取得が完了しました')
    }
    catch (e) {
      showErrorToast('サムネイルの再取得に失敗しました')
      console.error(e)
    }
    finally {
      disabledRefetchThumbnail = false
    }
  }
</script>

<VirtualScroller className='p-8'>
  {#snippet topElement()}
    <div class='space-y-8 mb-2'>
      <div class='flex items-center gap-2 w-full'>
        <img src={Icon} alt='launcherg icon' class='h-12' />
        <div class='font-logo text-(8 text-primary)'>Launcherg</div>
      </div>
      {#if $sidebarCollectionElements.length === 0 && isOpenGettingStarted}
        <div
          class='space-y-2 p-4 border-(border-primary solid ~) rounded max-w-120'
        >
          <div class='flex items-center'>
            <div class='text-(text-primary h3) font-medium'>Getting started</div>
          </div>
          <div class='text-(text-tertiary body)'>
            持っているゲームをこのランチャーに登録してみましょう。左のサイドバーにある「Add」ボタンから自動で追加できます。
          </div>
        </div>
      {/if}
      <div class='space-y-2'>
        <div class='text-(text-primary h3) font-medium'>Help</div>
        <LinkText
          href='https://youtu.be/GCTj6eRRgAM?si=WRFuBgNErwTJsNnk'
          text='1分でわかる Launcherg'
        />
        <LinkText
          href='https://ryoha000.hatenablog.com/entry/2023/09/24/003605'
          text='よくある Q&A'
        />
      </div>
      <div class='space-y-2'>
        <div class='text-(text-primary h3) font-medium'>Memo</div>
        {#await memoPromises then elements}
          {#if elements.length === 0 && $sidebarCollectionElements.length !== 0}
            <div
              class='space-y-2 p-4 border-(border-primary solid ~) rounded max-w-120'
            >
              <div class='flex items-center'>
                <div class='text-(text-primary h3) font-medium'>メモ機能</div>
              </div>
              <div class='text-(text-tertiary body)'>
                このアプリにはメモ機能があります。サイドバーからゲームを選択して「Memo」ボタンを押すことでそのゲームについてメモを取ることができます。
              </div>
            </div>
          {:else}
            <div class='gap-1 flex-(~ col)'>
              {#each elements as element (element.id)}
                <a
                  use:route
                  href='/memos/{element.id}?gamename={element.gamename}'
                  class='text-(text-link body2) hover:underline-(1px text-link)'
                >
                  メモ - {element.gamename}
                </a>
              {/each}
            </div>
          {/if}
        {/await}
      </div>
      <div class='flex items-center gap-4'>
        <h3 class='text-(text-primary h3) font-medium'>登録したゲーム</h3>
        <Button
          leftIcon='i-material-symbols-refresh-rounded'
          text='サムネイルを再取得する'
          disabled={disabledRefetchThumbnail}
          onclick={refetchThumbnail}
        />
      </div>
    </div>
  {/snippet}
  {#snippet children({ containerHeight, contentsWidth, contentsScrollY, setVirtualHeight, contentsScrollTo })}
    <VirtualScrollerMasonry
      elements={flattenShown}
      {setVirtualHeight}
      {contentsScrollY}
      {contentsWidth}
      {containerHeight}
      {contentsScrollTo}
    />
  {/snippet}
</VirtualScroller>
