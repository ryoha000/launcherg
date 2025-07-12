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
  on:close={() => (isOpen = false)}
  on:cancel={() => (isOpen = false)}
  title='Link to Smartphone'
  autofocusCloseButton
  withFooter={false}
>
  <div class='space-y-4 text-text-primary'>
    <div>
      QRコードを読み込む、またはリンクを共有することでほかの端末からメモを取れます
    </div>
    {#if readyPromise}
      {#await readyPromise}
        <div class='flex-(~ col) items-center justify-center gap-5 w-full p-12'>
          <div
            class='w-20 h-20 border-(12px solid #D9D9D9 t-#2D2D2D t-rounded) rounded-full animate-spin'
          ></div>
          <div class='text-(text-primary h3) font-bold'>処理中</div>
        </div>
      {:then value}
        <div class='flex-(~ col) justify-center items-center gap-4'>
          <button
            onclick={() => copyUrlToClipboard(value)}
            class='flex hover:bg-bg-button rounded px-4 py-1 items-center gap-4 bg-inherit'
          >
            <div
              class='i-material-symbols-content-copy-outline-rounded w-5 h-5'
            ></div>
            <div>{new URL(value).origin}?d=...</div>
          </button>
          <QrCodeCanvas {value} />
        </div>
      {/await}
    {/if}
  </div>
</Modal>
