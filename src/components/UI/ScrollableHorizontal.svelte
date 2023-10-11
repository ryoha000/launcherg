<svelte:options accessors />

<script lang="ts">
  import SimpleBar from "simplebar";
  import { createEventDispatcher } from "svelte";

  const dispatcher = createEventDispatcher<{ scroll: { event: Event } }>();

  let isHover = false;

  const simplebar = (node: HTMLElement) => {
    let simplebar = new SimpleBar(node, {
      scrollbarMinSize: 64,
    });

    const onScroll = (e: Event) => {
      dispatcher("scroll", { event: e });
    };
    simplebar.getScrollElement()?.addEventListener("scroll", onScroll);

    const onWheel = (e: WheelEvent) => {
      if (isHover) {
        simplebar
          .getScrollElement()
          ?.scrollBy({ left: e.deltaY * 5, behavior: "smooth" });
      }
    };
    window.addEventListener("wheel", onWheel);

    const element = simplebar.getScrollElement();
    if (element) {
      scrollBy = (options?: ScrollToOptions | undefined) => {
        element.scrollBy(options);
      };
    }
    return {
      destroy: () => {
        removeEventListener("wheel", onWheel);
        simplebar.getScrollElement()?.removeEventListener("scroll", onScroll);
        scrollBy = () => undefined;
      },
    };
  };

  export let scrollBy = (options?: ScrollToOptions | undefined): void => {
    console.warn("scrollBy is not initialized");
  };
</script>

<div class="w-full min-w-0">
  <div
    use:simplebar
    class="overflow-x-auto scroll-smooth"
    on:mouseenter={() => (isHover = true)}
    on:mouseleave={() => (isHover = false)}
  >
    <slot />
  </div>
</div>
