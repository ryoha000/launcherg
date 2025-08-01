<script lang='ts'>
  import OptionButton from '@/components/UI/OptionButton.svelte'

  let { onclose, onselectChange, onselectDelete, onselectOpen, onselectOtherInfomation }: {
    onclose: () => void
    onselectChange: () => void
    onselectDelete: () => void
    onselectOpen: () => void
    onselectOtherInfomation: () => void
  } = $props()
</script>

<div>
  <div
    class='flex items-center gap-4 p-(y-2 l-4 r-2) text-(body3 text-primary) font-bold'
  >
    <div class='whitespace-nowrap'>Select game option</div>
    <button
      onclick={onclose}
      class='i-iconoir-cancel ml-auto h-5 w-5 color-text-tertiary transition-all hover:color-text-primary'
      aria-label='Close options'
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
