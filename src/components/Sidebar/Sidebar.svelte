<script lang='ts'>
  import type { SortOrder } from '@/components/Sidebar/sort'
  import type { Option } from '@/lib/trieFilter'
  import { onMount } from 'svelte'
  import { fly } from 'svelte/transition'
  import CollectionElements from '@/components/Sidebar/CollectionElements.svelte'
  import Header from '@/components/Sidebar/Header.svelte'
  import MinimalSidebar from '@/components/Sidebar/MinimalSidebar.svelte'
  import { search } from '@/components/Sidebar/search'
  import Search from '@/components/Sidebar/Search.svelte'
  import { searchAttributes } from '@/components/Sidebar/searchAttributes'
  import SubHeader from '@/components/Sidebar/SubHeader.svelte'
  import { useFilter } from '@/lib/filter'
  import { collectionElementsToOptions } from '@/lib/trieFilter'
  import { createWritable, localStorageWritable } from '@/lib/utils'
  import { query } from '@/store/query'
  import { showSidebar } from '@/store/showSidebar'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'

  onMount(async () => {
    await sidebarCollectionElements.refetch()
  })

  const [elementOptions, getElementOptions] = createWritable<Option<number>[]>(
    [],
  )
  sidebarCollectionElements.subscribe(v =>
    elementOptions.set(collectionElementsToOptions(v)),
  )

  const { filtered } = useFilter(query, elementOptions, getElementOptions)
  const order = localStorageWritable<SortOrder>('sort-order', 'gamename-asc')
  const { attributes, toggleEnable } = searchAttributes()

  const shown = sidebarCollectionElements.shown
  filtered.subscribe(() => shown.set(search($filtered, $attributes, $order)))
  attributes.subscribe(() => shown.set(search($filtered, $attributes, $order)))
  order.subscribe(() => shown.set(search($filtered, $attributes, $order)))

  sidebarCollectionElements.subscribe(() => {
    shown.set(search($filtered, $attributes, $order))
  })
</script>

<div
  class='relative min-h-0 border-(r-1px border-primary solid) transition-all'
  class:w-80={$showSidebar}
  class:w-12={!$showSidebar}
>
  {#if $showSidebar}
    <div class='absolute inset-0' transition:fly={{ x: -40, duration: 150 }}>
      <div
        class='relative grid grid-(rows-[min-content_min-content_min-content_1fr]) h-full min-h-0 w-full'
      >
        <Header />
        <SubHeader />
        <div class='mt-2 w-full px-2'>
          <Search
            bind:query={$query}
            bind:order={$order}
            attributes={$attributes}
            on:toggleAttributeEnabled={e => toggleEnable(e.detail.key)}
          />
        </div>
        <div class='mt-1 min-h-0'>
          <CollectionElements collectionElement={$shown} />
        </div>
      </div>
    </div>
  {:else}
    <div class='absolute inset-0' transition:fly={{ x: 40, duration: 150 }}>
      <MinimalSidebar />
    </div>
  {/if}
</div>
