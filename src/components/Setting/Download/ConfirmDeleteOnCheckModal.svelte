<script lang='ts'>
  import { createEventDispatcher } from 'svelte'
  import Checkbox from '@/components/UI/Checkbox.svelte'
  import Modal from '@/components/UI/Modal.svelte'

  interface Props {
    isOpen: boolean
    targetTitle: string | null
    initialDontAskAgain?: boolean
  }

  const props: Props = $props()

  const dispatch = createEventDispatcher<{ confirm: { dontAskAgain: boolean }, cancel: void, close: void }>()

  let dontAskAgain = $state(props.initialDontAskAgain ?? false)

  $effect.pre(() => {
    dontAskAgain = props.initialDontAskAgain ?? false
  })

  function onConfirm() {
    dispatch('confirm', { dontAskAgain })
  }
  function onCancel() {
    dispatch('cancel')
  }
  function onClose() {
    dispatch('close')
  }
</script>

<Modal
  isOpen={props.isOpen}
  title='ライブラリから削除する'
  confirmText='削除する'
  cancelText='キャンセル'
  onconfirm={onConfirm}
  oncancel={onCancel}
  onclose={onClose}
>
  {#snippet children()}
    <div class='space-y-3'>
      <div>対象の項目はライブラリから削除され、以後の同期でも自動登録されません。PC上のファイルや購入履歴には影響しません。必要になったら設定を解除して再取り込みできます。</div>
      {#if props.targetTitle}
        <div class='text-(sm text-secondary)'>対象: {props.targetTitle}</div>
      {/if}
      <label class='flex items-center gap-2'>
        <Checkbox value={dontAskAgain} on:update={e => (dontAskAgain = e.detail.value)} />
        <span>次回からこのダイアログは表示せず、チェック時に自動で削除する</span>
      </label>
    </div>
  {/snippet}
</Modal>
