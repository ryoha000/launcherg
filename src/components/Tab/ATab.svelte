<script lang="ts">
  import { Tab } from "@rgossiaux/svelte-headlessui";
  import type { Tab as TTab } from "@/store/tabs";

  export let tab: TTab;

  $: tabIcon =
    tab.type === "works"
      ? "i-material-symbols-computer-outline-rounded color-accent-accent"
      : tab.type === "memos"
      ? "i-material-symbols-drive-file-rename-outline color-accent-edit"
      : "";
</script>

<Tab let:selected as="div">
  <div
    class="flex items-center gap-2 px-3 h-10 transition-all cursor-pointer border-(b-1px r-1px solid border-primary) group {selected
      ? 'bg-bg-primary border-b-transparent'
      : 'bg-bg-disabled hover:bg-bg-primary'}"
  >
    <div class="{tabIcon} w-5 h-5" />
    <div
      class="text-body2 whitespace-nowrap {selected
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
        on:click|stopPropagation={() => {}}
      />
    </div>
  </div>
</Tab>
