<script lang="ts">
  import Sidebar from "@/components/Sidebar/Sidebar.svelte";
  import ATabList from "@/components/Tab/ATabList.svelte";
  import { TabGroup } from "@rgossiaux/svelte-headlessui";
</script>

<div class="grid grid-cols-[300px_1fr] overflow-hidden w-full h-full">
  <Sidebar />
  <div
    class="h-full min-w-0 min-h-0 grid-(~ rows-[min-content_1fr]) overflow-hidden"
  >
    <TabGroup>
      <ATabList />
    </TabGroup>
    <div class="h-full min-h-0 min-w-0 w-full">
      <slot />
    </div>
  </div>
</div>
