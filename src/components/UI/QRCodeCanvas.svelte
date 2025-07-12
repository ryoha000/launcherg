<script lang='ts'>
  // @ts-ignore
  import QRious from 'qrious'
  import { onMount } from 'svelte'

  interface Props {
    value: string
  }

  const { value }: Props = $props()

  let canvas: HTMLCanvasElement | null = $state(null)

  onMount(() => {
    if (canvas) {
      new QRious({
        element: canvas,
        value,
        size: 400,
        background: '#f7f7f4',
      })
    }
  })
</script>

<canvas bind:this={canvas}></canvas>
