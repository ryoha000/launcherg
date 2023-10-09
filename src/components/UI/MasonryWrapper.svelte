<script lang="ts" generics="T">
  // 参考: https://www.webdesignleaves.com/pr/plugins/css-grid-masonry.html

  let container: HTMLDivElement | undefined;

  let rowHeight = 0;
  let rowGap = 0;

  $: {
    if (container) {
      rowHeight = +window
        .getComputedStyle(container)
        .getPropertyValue("grid-auto-rows")
        .replace("px", "");

      rowGap = +window
        .getComputedStyle(container)
        .getPropertyValue("grid-row-gap")
        .replace("px", "");
    }
  }
</script>

<div
  bind:this={container}
  class="grid gap-4 grid-cols-[repeat(auto-fill,_minmax(16rem,_1fr))] auto-rows-0"
>
  <slot {rowGap} {rowHeight} />
</div>
