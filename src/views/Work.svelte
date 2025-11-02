<script lang='ts'>
  import WorkErogameScape from '@/components/Work/WorkErogameScape.svelte'
  // import { commandGetCollectionElement, commandGetWorkDetailsByCollectionElement, commandOpenFolder, commandOpenUrl } from '@/lib/command'
  // import { useWorkDetailsQuery } from '@/lib/data/queries/workDetails'

  let { route }: { route?: { result: { path: { params: { id?: string } } } } } = $props()
  const idParam = $derived(route?.result?.path?.params?.id || '')
  const workId = $derived(idParam)

// async function openDmmDownload(dmm: { storeId: string, category: string, subcategory: string }) {
  //   const payload = {
  //     type: 'download',
  //     value: {
  //       game: {
  //         storeId: dmm.storeId,
  //         category: dmm.category,
  //         subcategory: dmm.subcategory,
  //       },
  //     },
  //   }
  //   const url = new URL('https://dlsoft.dmm.co.jp/mylibrary/')
  //   // URLSearchParams が自動でエンコードするため encodeURIComponent は不要
  //   url.searchParams.set('launcherg', JSON.stringify(payload))
  //   await commandOpenUrl(url.toString())
  // }

  // async function openDlsiteDownload(dlsite: { storeId: string, category: string }) {
  //   const payload = {
  //     type: 'download',
  //     value: {
  //       game: {
  //         storeId: dlsite.storeId,
  //         category: dlsite.category,
  //       },
  //     },
  //   }
  //   const url = new URL('https://play.dlsite.com/library')
  //   url.searchParams.set('launcherg', JSON.stringify(payload))
  //   await commandOpenUrl(url.toString())
  // }

  // async function openLatestDownloadFolder(path: string) {
  //   await commandOpenFolder(path)
  // }
</script>

<div class='h-full w-full'>
  <!-- {#if data.workDetail?.dmm}
    <div class='mt-4'>
      <button class='rounded bg-blue-600 px-3 py-1 text-white hover:bg-blue-700' onclick={() => openDmmDownload(data.workDetail!.dmm!)}>
        DMMからダウンロード
      </button>
    </div>
  {/if}
  {#if data.workDetail?.dlsite}
    <div class='mt-2'>
      <button class='rounded bg-green-600 px-3 py-1 text-white hover:bg-green-700' onclick={() => openDlsiteDownload(data.workDetail!.dlsite!)}>
        DLsiteからダウンロード
      </button>
    </div>
  {/if}
  {#if data.workDetail?.latestDownloadPath?.downloadPath}
    <div class='mt-2'>
      <button class='rounded bg-gray-600 px-3 py-1 text-white hover:bg-gray-700' onclick={() => openLatestDownloadFolder(data.workDetail!.latestDownloadPath!.downloadPath)}>
        最新ダウンロードフォルダを開く
      </button>
    </div>
  {/if} -->
  <WorkErogameScape {workId} />
</div>
