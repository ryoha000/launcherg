<script lang='ts'>
  import OptionButton from '@/components/UI/OptionButton.svelte'

  const { onclose, onstartAuto, onstartManual }: { onclose: () => void, onstartAuto: () => void, onstartManual: () => void } = $props()
</script>

<div>
  <div
    class='font-bold text-(text-primary body3) p-(l-4 r-2 y-2) flex items-center gap-4'
  >
    <div class='whitespace-nowrap'>Select adding game option</div>
    <button
      onclick={onclose}
      class='ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all'
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
