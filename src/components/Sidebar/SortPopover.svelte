<script lang='ts'>
  import type { SortOrder } from '@/components/Sidebar/sort'
  import { SORT_LABELS } from '@/components/Sidebar/sort'
  import OptionButton from '@/components/UI/OptionButton.svelte'

  interface Props {
    value: SortOrder
    onclose: () => void
  }

  let { value = $bindable(), onclose }: Props = $props()

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
    class='flex items-center p-(y-2 l-4 r-2) text-(body3 text-primary) font-bold'
  >
    <div>Select sort option</div>
    <button
      onclick={onclose}
      class='i-iconoir-cancel ml-auto h-5 w-5 color-text-tertiary transition-all hover:color-text-primary'
      aria-label='Close options'
    ></button>
  </div>
  {#each sortOrders as sortOrder (sortOrder)}
    <OptionButton
      onclick={() => {
        value = sortOrder
        onclose()
      }}
      selected={value === sortOrder}
      text={SORT_LABELS[sortOrder]}
      showIcon
    />
  {/each}
</div>
