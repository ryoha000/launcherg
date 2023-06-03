<script lang="ts">
  import Sidebar from "@/components/Sidebar/Sidebar.svelte";
  import ATabList from "@/components/Tab/ATabList.svelte";
  import { TabGroup } from "@rgossiaux/svelte-headlessui";
</script>

<div class="grid grid-cols-[300px_1fr] overflow-hidden w-full h-full">
  <Sidebar />
  <div class="h-full min-w-0">
    <TabGroup>
      <ATabList />
    </TabGroup>
    <slot />
  </div>
</div>
