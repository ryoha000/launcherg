<script lang="ts">
  import SimpleBar from "simplebar";

  let isHover = false;

  const simplebar = (node: HTMLElement) => {
    let simplebar = new SimpleBar(node, {
      scrollbarMinSize: 64,
    });

    const onWheel = (e: WheelEvent) => {
      if (isHover) {
        simplebar
          .getScrollElement()
          ?.scrollBy({ left: e.deltaY * 5, behavior: "smooth" });
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
