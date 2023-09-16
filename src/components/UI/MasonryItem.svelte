<script lang="ts">
  export let rowHeight: number;
  export let rowGap: number;

  let container: HTMLDivElement | undefined;
  let content: HTMLDivElement | undefined;

  const onResize = () => {
    if (!container || !content) {
      console.error("MasonryItem.(container|content) is undefined");
      return;
    }

    const rowSpan = Math.ceil(
      (content.getBoundingClientRect().height + rowGap) / (rowHeight + rowGap)
    );
    container.style.gridRowEnd = `span ${rowSpan}`;
  };

  const resizeObserver = new ResizeObserver(onResize);
  $: {
    onResize();
    if (content) {
      resizeObserver.observe(content);
    }
  }
</script>

<div bind:this={container}>
  <div bind:this={content}><slot /></div>
</div>
