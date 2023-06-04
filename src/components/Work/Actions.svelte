<script lang="ts">
  import PlayButton from "@/components/Work/PlayButton.svelte";
</script>

<div class="flex items-center gap-4 flex-wrap w-full min-w-0">
  <PlayButton />
  <button
    class="bg-accent-edit min-w-0 rounded transition-all hover:bg-opacity-80 flex items-center h-8 px-4 gap-2"
  >
    <div
      class="w-4 h-4 color-text-secondary i-material-symbols-drive-file-rename-outline"
    />
    <div class="text-(text-secondary body2) font-medium">Memo</div>
  </button>
  <div class="flex ml-auto items-end gap-2 h-8 min-w-0">
    <div class="text-(text-primary body2) whitespace-nowrap">Time</div>
    <div class="text-(text-primary body)">03:22</div>
  </div>
</div>
