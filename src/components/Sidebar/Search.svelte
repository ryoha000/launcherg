<script lang='ts'>
  import type { Attribute, AttributeKey } from '@/components/Sidebar/searchAttributes'
  import type { SortOrder } from '@/components/Sidebar/sort'
  import { createEventDispatcher } from 'svelte'
  import SearchAttribute from '@/components/Sidebar/SearchAttribute.svelte'
  import SearchAttrributeControl from '@/components/Sidebar/SearchAttrributeControl.svelte'
  import SearchInput from '@/components/Sidebar/SearchInput.svelte'
  import SortPopover from '@/components/Sidebar/SortPopover.svelte'
  import APopover from '@/components/UI/APopover.svelte'
  import ButtonBase from '@/components/UI/ButtonBase.svelte'
  import ScrollableHorizontal from '@/components/UI/ScrollableHorizontal.svelte'

  interface Props {
    query: string
    order: SortOrder
    attributes: Attribute[]
  }

  let { query = $bindable(), order = $bindable(), attributes }: Props = $props()

  const dispatcher = createEventDispatcher<{
    toggleAttributeEnabled: { key: AttributeKey }
  }>()

  let isShowBack = $state(false)
  let isShowForward = $state(true)
  const onScroll = (e: Event) => {
    const element = e.target as HTMLElement
    const rect = element.getBoundingClientRect()
    const width = element.scrollWidth

    const left = element.scrollLeft
    const right = width - rect.width - left

    isShowBack = left > 0
    isShowForward = right > 0
  }

  let scrollable: ScrollableHorizontal | undefined = $state()
</script>

<div class='w-full space-y-1'>
  <div class='flex items-center gap-2'>
    <div class='flex-1'>
      <SearchInput
        bind:value={query}
        placeholder='Filter by title, brand and more'
      />
    </div>
    <APopover panelClass='right-0'>
      {#snippet button({ open })}
        <ButtonBase
          appendClass='h-8 w-8 flex items-center justify-center'
          tooltip={open
            ? undefined
            : {
              content: 'ゲームの並べ替え',
              placement: 'bottom',
              theme: 'default',
              delay: 1000,
            }}
        >
          <div
            class='i-material-symbols-sort-rounded h-5 w-5 color-ui-tertiary'
          ></div>
        </ButtonBase>
      {/snippet}
      {#snippet children({ close })}
        <SortPopover bind:value={order} onclose={close} />
      {/snippet}
    </APopover>
  </div>
  <div class='hide-scrollbar relative'>
    <ScrollableHorizontal
      onscroll={onScroll}
      bind:this={scrollable}
    >
      <div class='flex items-center gap-2 pb-1'>
        {#each attributes as attribute (attribute.key)}
          <SearchAttribute
            {attribute}
            onclick={() =>
              dispatcher('toggleAttributeEnabled', { key: attribute.key })}
          />
        {/each}
      </div>
    </ScrollableHorizontal>
    <SearchAttrributeControl
      appendClass='left-0'
      back
      show={isShowBack}
      onclick={() => scrollable?.scrollBy({ left: -100, behavior: 'smooth' })}
    />
    <SearchAttrributeControl
      appendClass='right-0'
      show={isShowForward}
      onclick={() => scrollable?.scrollBy({ left: 100, behavior: 'smooth' })}
    />
  </div>
</div>
