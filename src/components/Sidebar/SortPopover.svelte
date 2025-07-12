<script lang='ts'>
  import type { SortOrder } from '@/components/Sidebar/sort'
  import { createEventDispatcher } from 'svelte'
  import { SORT_LABELS } from '@/components/Sidebar/sort'
  import OptionButton from '@/components/UI/OptionButton.svelte'

  interface Props {
    value: SortOrder
  }

  let { value = $bindable() }: Props = $props()
  const dispatcher = createEventDispatcher<{ close: {} }>()

  const sortOrders: SortOrder[] = [
    'gamename-asc',
    'gamename-desc',
    'sellyear-desc',
    'install-desc',
    'last_play-desc',
    'registered-desc',
    'brandname-asc',
    'brandname-desc',
  ]
</script>

<div>
  <div
    class='font-bold text-(text-primary body3) p-(l-4 r-2 y-2) flex items-center'
  >
    <div>Select sort option</div>
    <button
      onclick={() => dispatcher('close')}
      class='ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all'
    ></button>
  </div>
  {#each sortOrders as sortOrder (sortOrder)}
    <OptionButton
      on:click={() => {
        value = sortOrder
        dispatcher('close')
      }}
      selected={value === sortOrder}
      text={SORT_LABELS[sortOrder]}
      showIcon
    />
  {/each}
</div>
