<script lang="ts">
  import { onMount } from "svelte";
  // @ts-ignore
  import QRious from "qrious";
  export let value: string;

  let canvas: HTMLCanvasElement | null = null;

  onMount(() => {
    if (canvas) {
      new QRious({
        element: canvas,
        value,
        size: 400,
        background: "#f7f7f4",
      });
    }
  });
</script>

<canvas bind:this={canvas} />
