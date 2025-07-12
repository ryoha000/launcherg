<script lang='ts'>
  import type { CollectionElement } from '@/lib/types'
  import Button from '@/components/UI/Button.svelte'
  import Modal from '@/components/UI/Modal.svelte'
  import { commandDeleteCollectionElement } from '@/lib/command'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'
  import { deleteTab, selected, tabs } from '@/store/tabs'

  interface Props {
    isOpen: boolean
    element: CollectionElement
  }

  let { isOpen = $bindable(), element }: Props = $props()

  const deleteGame = async () => {
    await commandDeleteCollectionElement(element.id)
    await sidebarCollectionElements.refetch()
    deleteTab($tabs[$selected].id)
    isOpen = false
  }
</script>

<Modal
  {isOpen}
  onclose={() => (isOpen = false)}
  oncancel={() => (isOpen = false)}
  title='Delete game'
  withContentPadding={false}
  autofocusCloseButton
  headerClass='border-b-(border-warning opacity-40) '
>
  <div
    class='bg-bg-warning border-(b-1px solid border-warning opacity-40) flex gap-2 p-(x-4 y-5)'
  >
    <div
      class='w-6 h-6 i-material-symbols-warning-outline-rounded color-accent-warning'
    ></div>
    <div class='space-y-1'>
      <div class='text-(body text-primary) font-medium'>
        このゲームの登録を削除します
      </div>
      <div class='text-(body2 text-primary)'>
        参照元のファイルが消えることはありません。プレイ時間のデータは同じゲームを登録したとき引き継がれます。
      </div>
    </div>
  </div>
  {#snippet footer()}
    <div class='p-4 max-w-full'>
      <Button
        text='{element.gamename} を削除する'
        variant='error'
        wrappable
        appendClass='w-full justify-center'
        onclick={deleteGame}
      />
    </div>
  {/snippet}
</Modal>
