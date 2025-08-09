<script lang='ts'>
  import type { Tab } from '@/store/tabs'

  import { goto } from '@mateothegreat/svelte5-router'
  import { deleteTab } from '@/store/tabs'

  interface Props {
    tab: Tab
    selected: boolean
  }

  const { tab, selected }: Props = $props()

  const tabIcon
    = $derived(tab.type === 'works'
      ? 'i-material-symbols-computer-outline-rounded color-accent-accent'
      : tab.type === 'memos'
      ? 'i-material-symbols-drive-file-rename-outline color-accent-edit'
      : tab.type === 'settings'
      ? 'i-material-symbols-settings-outline-rounded color-text-disabled'
      : '')

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
    if (tab.type === 'settings') {
      goto('/settings')
    }
    else if (tab.type.startsWith('debug-')) {
      goto(`/debug/${tab.type.split('-')[1]}`)
    }
    else {
      goto(`/${tab.type}/${tab.workId}`)
    }
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
