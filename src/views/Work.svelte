<script lang='ts'>
  import type { Work } from '@/lib/types'
  import WorkErogameScape from '@/components/Work/WorkErogameScape.svelte'
  import { commandGetCollectionElement } from '@/lib/command'
  import { works } from '@/store/works'

  let { route }: { route?: { result: { path: { params: { id?: string } } } } } = $props()
  let idParam = $derived(route?.result?.path?.params?.id || '')

  // ビューの責務: 表示ソース判定とデータ取得
  let viewPromise = $derived((async () => {
    const collectionElement = await commandGetCollectionElement(+idParam)
    const erogameScapeId = collectionElement.erogamescapeId ?? null
    let work: Work | null = null
    if (erogameScapeId) {
      work = (await works.get(erogameScapeId)) ?? null
    }
    return { collectionElement, work }
  })())
</script>

{#await viewPromise}
  <div class='h-full w-full flex items-center justify-center'>
    <span class='text-gray-500'>Loading...</span>
  </div>
{:then data}
  {#if data.work}
    <div class='h-full w-full'>
      <WorkErogameScape work={data.work} />
    </div>
  {:else}
    <div class='h-full w-full p-4'>
      <div class='text-gray-500'>TODO: erogamescape id が紐づいていない。何らかの情報をソースに何かを書く {JSON.stringify(data.collectionElement)}</div>
    </div>
  {/if}
{:catch error}
  <div class='h-full w-full flex items-center justify-center'>
    <span class='text-red-500'>Error: {error.message}</span>
  </div>
{/await}
