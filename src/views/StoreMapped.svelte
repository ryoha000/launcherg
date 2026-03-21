<script lang='ts'>
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'
  import { get } from 'svelte/store'
  import dlsiteIconUrl from '@/assets/dlsite.ico'
  import dmmIconUrl from '@/assets/dmm.ico'
  import egsIconUrl from '@/assets/erogamescape.ico'
  import APopover from '@/components/UI/APopover.svelte'
  import Button from '@/components/UI/Button.svelte'
  import LinkServiceFilter from '@/components/WorkList/LinkServiceFilter.svelte'
  import { useWorkDetailsAllQuery } from '@/lib/data/queries/workDetails'

  const workDetailsQuery = useWorkDetailsAllQuery()

  // ストア別三段階フィルタ（any / linked / unlinked）
  type FilterMode = 'any' | 'linked' | 'unlinked'
  type StoreFilter = { dmm: FilterMode, dlsite: FilterMode, egs: FilterMode }
  let storeFilter: StoreFilter = $state({ dmm: 'any', dlsite: 'any', egs: 'any' })
  const modeLabel = (m: FilterMode) => (m === 'linked' ? 'あり' : m === 'unlinked' ? 'なし' : '全て')
  const storeFilterLabel = $derived.by(() => {
    const allAny = storeFilter.dmm === 'any' && storeFilter.dlsite === 'any' && storeFilter.egs === 'any'
    if (allAny)
      return 'すべて表示'
    const parts: string[] = []
    if (storeFilter.dmm !== 'any')
      parts.push(`DMM:${modeLabel(storeFilter.dmm)}`)
    if (storeFilter.dlsite !== 'any')
      parts.push(`DLsite:${modeLabel(storeFilter.dlsite)}`)
    if (storeFilter.egs !== 'any')
      parts.push(`批評空間:${modeLabel(storeFilter.egs)}`)
    return parts.length > 0 ? parts.join(', ') : 'すべて表示'
  })
  let keyword = $state('')

  // 表示件数系（WorkDetails を直接参照）
  const items = $derived.by(() => ($workDetailsQuery.data ?? []))
  const totalCount = $derived.by(() => (items.length))
  const dmmCount = $derived.by(() => (items.filter(w => !!w.dmm).length))
  const dlsiteCount = $derived.by(() => (items.filter(w => !!w.dlsite).length))

  const filteredItems = $derived.by(() => {
    const q = keyword.trim().toLowerCase()
    return items.filter((w) => {
      const matchDmm = storeFilter.dmm === 'any' || (storeFilter.dmm === 'linked' ? !!w.dmm : !w.dmm)
      const matchDl = storeFilter.dlsite === 'any' || (storeFilter.dlsite === 'linked' ? !!w.dlsite : !w.dlsite)
      const matchEgs = storeFilter.egs === 'any' || (storeFilter.egs === 'linked' ? !!w.erogamescapeId : !w.erogamescapeId)
      if (!(matchDmm && matchDl && matchEgs))
        return false
      if (!q)
        return true
      const inTitle = w.title.toLowerCase().includes(q)
      return inTitle
    })
  })

  onMount(async () => {
    await get(workDetailsQuery).refetch()
  })
</script>

<div class='grid grid-(rows-[auto_auto_auto_auto_1fr]) h-full w-full p-4'>
  <div class='mb-2 text-(h3 text-primary)'>ダウンロード購入作品の管理</div>
  <div class='mb-3 text-(sm text-secondary) -mt-1'>
    取り込み内容を随時見直し、不要な項目を適切に整理できます。<br />
    連携状況を確認しながら、作品一覧を整理できます。
  </div>
  <div class='mb-1 flex items-center gap-3'>
    <APopover panelClass='left-0 min-w-56'>
      {#snippet button()}
        <Button
          text={storeFilterLabel}
          rightIcon='i-material-symbols-arrow-drop-down-rounded'
        />
      {/snippet}
      {#snippet children()}
        <div class='p-2'>
          <LinkServiceFilter bind:filter={storeFilter} />
        </div>
      {/snippet}
    </APopover>
    <input
      class='ml-2 max-w-xs w-full border border-(border-primary) rounded bg-(bg-primary) p-2 text-(text-primary)'
      placeholder='タイトル検索'
      bind:value={keyword}
    />
    <div class='ml-auto text-(sm text-secondary)'>
      全 {totalCount} 件
      <span class='ml-3'>DMM {dmmCount} 件</span>
      <span class='ml-2'>DLsite {dlsiteCount} 件</span>
    </div>
  </div>
  <div class='overflow-hidden border-(1px border-primary solid) rounded'>
    <div class='max-h-full overflow-auto'>
      <table class='w-full border-separate border-spacing-0 table-fixed whitespace-nowrap text-(left text-primary)'>
        <thead class='sticky top-0 z-20 bg-bg-primary'>
          <tr>
            <th class='w-24 border-(b border-primary) px-2 py-2'>連携</th>
            <th class='w-18 border-(b border-primary) px-2 py-2'></th>
            <th class='w-36 border-(b border-primary) px-2 py-2'>タイトル</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredItems as item}
            <tr class='border-(b border-primary solid)'>
              <td class='px-2 py-1'>
                <div class='flex items-center gap-2'>
                  <div class='h-5 w-5'>
                    {#if item.dmm}
                      <img src={dmmIconUrl} alt='DMM' class='h-5 w-5 object-contain' />
                    {/if}
                  </div>
                  <div class='h-5 w-5'>
                    {#if item.dlsite}
                      <img src={dlsiteIconUrl} alt='DLsite' class='h-5 w-5 object-contain' />
                    {/if}
                  </div>
                  <div class='h-5 w-5'>
                    {#if item.erogamescapeId}
                      <img src={egsIconUrl} alt='EGS' class='h-5 w-5 object-contain' />
                    {/if}
                  </div>
                </div>
              </td>
              <td class='px-2 py-1'>
                {#if item.thumbnail}
                  <div class='h-12 w-20 overflow-hidden rounded bg-bg-secondary'>
                    <img src={convertFileSrc(item.thumbnail.path)} alt='thumbnail' class='h-full w-full object-cover' />
                  </div>
                {:else}
                  <div class='h-full w-full'>
                    <div class='h-full w-full'></div>
                  </div>
                {/if}
              </td>
              <td class='w-36 overflow-hidden text-ellipsis whitespace-nowrap px-2 py-1'>{item.title}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>
