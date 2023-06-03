<script lang="ts">
  import ATab from "@/components/Tab/ATab.svelte";
  import { tabs } from "@/store/tabs";
  import { TabList } from "@rgossiaux/svelte-headlessui";
  import SimpleBar from "simplebar";

  let isHover = false;

  const simplebar = (node: HTMLElement) => {
    let simplebar = new SimpleBar(node, { scrollbarMinSize: 64 });

    const onWheel = (e: WheelEvent) => {
      if (isHover) {
        simplebar
          .getScrollElement()
          ?.scrollBy({ left: e.deltaY * 2, behavior: "smooth" });
      }
    };
    window.addEventListener("wheel", onWheel);
    return {
      destroy: () => {
        removeEventListener("wheel", onWheel);
      },
    };
  };
</script>

<TabList class="w-full">
  <div
    use:simplebar
    class="overflow-x-auto scroll-smooth"
    on:mouseenter={() => (isHover = true)}
    on:mouseleave={() => (isHover = false)}
  >
    <div class="grid-(~ cols-[min-content_1fr]) items-center">
      <div class="flex items-center">
        {#each $tabs as tab (tab.id)}
          <ATab {tab} />
        {/each}
      </div>
      <div
        class="w-full h-full bg-bg-disabled border-(b-1px solid border-primary)"
      />
    </div>
  </div>
</TabList>
