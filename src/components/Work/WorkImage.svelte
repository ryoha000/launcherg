<script lang='ts'>
  interface Props {
    name: string
    src: string
  }

  const { name, src }: Props = $props()
</script>

<img alt='{name}_icon' {src} class='w-full object-contain' />
