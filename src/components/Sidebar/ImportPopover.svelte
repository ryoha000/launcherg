<script lang='ts'>
  import OptionButton from '@/components/UI/OptionButton.svelte'

  const { onclose, onstartAuto, onstartManual }: { onclose: () => void, onstartAuto: () => void, onstartManual: () => void } = $props()
</script>

<div>
  <div
    class='flex items-center gap-4 p-(y-2 l-4 r-2) text-(body3 text-primary) font-bold'
  >
    <div class='whitespace-nowrap'>Select adding game option</div>
    <button
      onclick={onclose}
      class='i-iconoir-cancel ml-auto h-5 w-5 color-text-tertiary transition-all hover:color-text-primary'
      aria-label='Close options'
    ></button>
  </div>
  <OptionButton
    text='自動でフォルダから追加'
    onclick={() => {
      onclose()
      onstartAuto()
    }}
  />
  <OptionButton
    text='手動で追加'
    onclick={() => {
      onclose()
      onstartManual()
    }}
  />
</div>
