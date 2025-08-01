<script lang='ts'>
  import { run } from 'svelte/legacy'

  import Modal from '@/components/UI/Modal.svelte'
  import QrCodeCanvas from '@/components/UI/QRCodeCanvas.svelte'
  import { showInfoToast } from '@/lib/toast'
  import { skyWay } from '@/store/skyway'

  interface Props {
    isOpen: boolean
    id: number
    seiyaUrl: string
  }

  let { isOpen = $bindable(), id, seiyaUrl }: Props = $props()

  let readyPromise: Promise<string> | undefined = $state(undefined)
  run(() => {
    if (isOpen) {
      readyPromise = skyWay.connect(id, seiyaUrl)
    }
  })

  const copyUrlToClipboard = async (value: string) => {
    await navigator.clipboard.writeText(value)
    showInfoToast('クリップボードにコピーしました')
  }
</script>

<Modal
  {isOpen}
  onclose={() => (isOpen = false)}
  oncancel={() => (isOpen = false)}
  title='Link to Smartphone'
  autofocusCloseButton
  withFooter={false}
  fullmodal
>
  <div class='text-text-primary space-y-4'>
    <div>
      QRコードを読み込む、またはリンクを共有することでほかの端末からメモを取れます
    </div>
    {#if readyPromise}
      {#await readyPromise}
        <div class='w-full flex flex-(col) items-center justify-center gap-5 p-12'>
          <div
            class='border-t-rounded h-20 w-20 animate-spin border-(12px #D9D9D9 t-#2D2D2D solid) rounded-full'
          ></div>
          <div class='text-(h3 text-primary) font-bold'>処理中</div>
        </div>
      {:then value}
        <div class='flex flex-(col) items-center justify-center gap-4'>
          <button
            onclick={() => copyUrlToClipboard(value)}
            class='flex items-center gap-4 rounded bg-inherit px-4 py-1 hover:bg-bg-button'
          >
            <div
              class='i-material-symbols-content-copy-outline-rounded h-5 w-5'
            ></div>
            <div>{new URL(value).origin}?d=...</div>
          </button>
          <QrCodeCanvas {value} />
        </div>
      {/await}
    {/if}
  </div>
</Modal>
