<script lang='ts'>
  import OptionButton from '@/components/UI/OptionButton.svelte'

  let { onclose, onselectChange, onselectDelete, onselectOpen, onselectOtherInfomation }: { 
    onclose: () => void,
    onselectChange: () => void,
    onselectDelete: () => void,
    onselectOpen: () => void,
    onselectOtherInfomation: () => void
  } = $props()
</script>

<div>
  <div
    class='font-bold text-(text-primary body3) p-(l-4 r-2 y-2) flex items-center gap-4'
  >
    <div class='whitespace-nowrap'>Select game option</div>
    <button
      onclick={onclose}
      class='ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all'
      aria-label="Close options"
    ></button>
  </div>
  <OptionButton
    text='ゲームのフォルダーを開く'
    onclick={() => {
      onclose()
      onselectOpen()
    }}
  />
  <OptionButton
    text='違うゲームに変更する'
    onclick={() => {
      onclose()
      onselectChange()
    }}
  />
  <OptionButton
    text='登録したゲームから削除する'
    onclick={() => {
      onclose()
      onselectDelete()
    }}
  />
  <OptionButton
    text='その他の情報'
    onclick={() => {
      onclose()
      onselectOtherInfomation()
    }}
  />
</div>
