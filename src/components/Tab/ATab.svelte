<script lang="ts">
  import { deleteTab, type Tab } from "@/store/tabs";
  import { push } from "svelte-spa-router";

  export let tab: Tab;
  export let selected: boolean;

  $: tabIcon =
    tab.type === "works"
      ? "i-material-symbols-computer-outline-rounded color-accent-accent"
      : tab.type === "memos"
      ? "i-material-symbols-drive-file-rename-outline color-accent-edit"
      : "";

  const closeWheelClick = (e: MouseEvent) => {
    if (e.button === 1) {
      deleteTab(tab.id);
    }
  };
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
  on:click={() => push(`/${tab.type}/${tab.workId}`)}
  on:mousedown={closeWheelClick}
>
  <div
    class="flex items-center gap-2 px-3 h-10 transition-all cursor-pointer border-(b-1px r-1px solid border-primary) group max-w-60 {selected
      ? 'bg-bg-primary border-b-transparent'
      : 'bg-bg-disabled hover:bg-bg-primary'}"
  >
    <div class="{tabIcon} w-5 h-5 flex-shrink-0" />
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
        on:click|stopPropagation={() => deleteTab(tab.id)}
      />
    </div>
  </div>
</div>
