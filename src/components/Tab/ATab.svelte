<script lang="ts">
  import { stopPropagation } from 'svelte/legacy';

  import { deleteTab, type Tab } from "@/store/tabs";
  import { goto } from "@mateothegreat/svelte5-router";

  interface Props {
    tab: Tab;
    selected: boolean;
  }

  let { tab, selected }: Props = $props();

  let tabIcon =
    $derived(tab.type === "works"
      ? "i-material-symbols-computer-outline-rounded color-accent-accent"
      : tab.type === "memos"
        ? "i-material-symbols-drive-file-rename-outline color-accent-edit"
        : "");

  const closeWheelClick = (e: MouseEvent) => {
    if (e.button === 1) {
      deleteTab(tab.id);
    }
  };
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  onclick={() => goto(`/${tab.type}/${tab.workId}`)}
  onmousedown={closeWheelClick}
>
  <div
    class="flex items-center gap-2 px-3 h-10 transition-all cursor-pointer border-(b-1px r-1px solid border-primary) group max-w-60 {selected
      ? 'bg-bg-primary border-b-transparent'
      : 'bg-bg-disabled hover:bg-bg-primary'}"
  >
    <div class="{tabIcon} w-5 h-5 flex-shrink-0"></div>
    <div
      class="text-body2 whitespace-nowrap text-ellipsis overflow-hidden {selected
        ? 'text-text-primary'
        : 'text-text-tertiary'}"
    >
      {tab.title}
    </div>
    <div
      class="rounded hover:bg-bg-secondary flex items-center justify-center transition-all"
    >
      <button
        class="group-hover:opacity-100 opacity-0 transition-all w-5 h-5 i-iconoir-cancel {selected
          ? 'color-text-secondary'
          : 'color-text-tertiary'}"
        onclick={stopPropagation(() => deleteTab(tab.id))}
></button>
    </div>
  </div>
</div>
