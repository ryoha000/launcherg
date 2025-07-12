<script lang='ts'>
  // @ts-ignore
  import QRious from 'qrious'

  interface Props {
    value: string
  }

  const { value }: Props = $props()

  let canvas: HTMLCanvasElement | null = $state(null)
  let qrios: QRious | undefined

  $effect(() => {
    if (value && canvas) {
      if (qrios) {
        qrios.set({ value })
      }
      else {
        qrios = new QRious({
          element: canvas,
          value,
          size: 400,
          background: '#f7f7f4',
        })
      }
    }
  })
</script>

<canvas bind:this={canvas}></canvas>
