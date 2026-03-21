<script lang='ts'>
  import type { Tab } from '@/store/tabs'

  import { goto } from '@mateothegreat/svelte5-router'
  import { ROUTE_REGISTRY } from '@/router/const'
  import { deleteTab } from '@/store/tabs'
  import { buildPath } from '@/store/tabs/schema'

  interface Props {
    tab: Tab
    selected: boolean
  }

  const { tab, selected }: Props = $props()

  const tabIcon = $derived.by(() => {
    const descriptor = ROUTE_REGISTRY.find(r => r.kind === tab.type)
    return descriptor && 'icon' in descriptor && descriptor.icon
  })

  const closeWheelClick = (e: MouseEvent) => {
    if (e.button === 1) {
      deleteTab(tab.id)
    }
  }

  const onClickCloseTabButton = (e: MouseEvent) => {
    e.stopPropagation()
    deleteTab(tab.id)
  }

  const navigate = () => {
    const d = ROUTE_REGISTRY.find(r => r.kind === tab.type)
    if (tab.href)
      return goto(tab.href)
    if (!d)
      return goto('/')
    goto(buildPath(d, tab.workId))
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  onclick={navigate}
  onmousedown={closeWheelClick}
>
  <div
    class="group  h-10 max-w-60 flex cursor-pointer items-center gap-2 border-(b-1px r-1px border-primary solid) px-3 transition-all {selected
      ? 'border-b-transparent bg-bg-primary'
      : 'bg-bg-disabled hover:bg-bg-primary'}"
  >
    <div class='{tabIcon} h-5 w-5 flex-shrink-0'></div>
    <div
      class="overflow-hidden text-ellipsis whitespace-nowrap text-body2 {selected
        ? 'text-text-primary'
        : 'text-text-tertiary'}"
    >
      {tab.title}
    </div>
    <div
      class='flex items-center justify-center rounded transition-all hover:bg-bg-secondary'
    >
      <button
        class="i-iconoir-cancel  h-5 w-5 opacity-0 transition-all group-hover:opacity-100 {selected
          ? 'color-text-secondary'
          : 'color-text-tertiary'}"
        onclick={onClickCloseTabButton}
        aria-label='Close tab'
      ></button>
    </div>
  </div>
</div>
