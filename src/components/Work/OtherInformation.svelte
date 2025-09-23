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
            {#if workDetail.dmm}
              <div>{workDetail.dmm.storeId}/{workDetail.dmm.category}/{workDetail.dmm.subcategory}</div>
            {:else}
              <div>未連携</div>
            {/if}
          </div>
          <div>
            <div>DLsite</div>
            {#if workDetail.dlsite}
              <div>{workDetail.dlsite.storeId}/{workDetail.dlsite.category}</div>
            {:else}
              <div>未連携</div>
            {/if}
          </div>
        </div>
      {/if}
    </OtherInfomationSection>
    <OtherInfomationSection label='Thumbnail Path'>
      <pre class='whitespace-pre-wrap break-all'>{workDetail.thumbnail}</pre>
    </OtherInfomationSection>
    <OtherInfomationSection label='Latest Download Path'>
      <pre class='whitespace-pre-wrap break-all'>{workDetail.latestDownloadPath?.downloadPath}</pre>
    </OtherInfomationSection>
  </div>
</Modal>
