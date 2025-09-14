<script lang='ts'>
  import type { WorkDetailsVm } from '@/lib/command'
  import Modal from '@/components/UI/Modal.svelte'
  import OtherInfomationSection from '@/components/Work/OtherInfomationSection.svelte'

  interface Props {
    isOpen: boolean
    workDetail: WorkDetailsVm
  }

  let { isOpen = $bindable(), workDetail }: Props = $props()
</script>

<Modal
  {isOpen}
  onclose={() => (isOpen = false)}
  oncancel={() => (isOpen = false)}
  title='Infomation'
  autofocusCloseButton
  withFooter={false}
>
  <div class='space-y-4'>
    <OtherInfomationSection label='Work ID' value={workDetail.id} />
    <OtherInfomationSection label='Title' value={workDetail.title} />
    <OtherInfomationSection label='ErogameScape ID' value={workDetail.erogamescapeId ?? 'EGS ID未連携'} />
    <OtherInfomationSection label='DMM/DLsite 購入履歴連携'>
      {#if !workDetail.dmm && !workDetail.dlsite}
        <div>購入履歴連携で登録されたゲームではありません</div>
      {:else}
        <div class='grid grid-cols-2 gap-2'>
          <div>
            <div>DMM</div>
            <div>{workDetail.dmm?.storeId}</div>
          </div>
          <div>
            <div>DLsite</div>
            <div>{workDetail.dlsite?.storeId}</div>
          </div>
        </div>
      {/if}
    </OtherInfomationSection>
    <OtherInfomationSection label='Thumbnail URL' value={workDetail.thumbnail} />
    <OtherInfomationSection label='Latest Download Path' value={workDetail.latestDownloadPath?.downloadPath} />
  </div>
</Modal>
