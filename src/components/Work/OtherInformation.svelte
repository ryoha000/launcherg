<script lang='ts'>
  import type { CollectionElement } from '@/lib/types'
  import Modal from '@/components/UI/Modal.svelte'
  import OtherInfomationSection from '@/components/Work/OtherInfomationSection.svelte'
  import {
    commandGetGameCacheById,
  } from '@/lib/command'

  interface Props {
    isOpen: boolean
    element: CollectionElement
  }

  let { isOpen = $bindable(), element }: Props = $props()

  const gameCache = $derived(
    element.erogamescapeId != null && element.erogamescapeId !== undefined
      ? commandGetGameCacheById(element.erogamescapeId)
      : Promise.resolve(null),
  )
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
    <OtherInfomationSection label='ErogameScape ID' value={element.erogamescapeId ?? 'EGS ID未連携'} />
    <OtherInfomationSection label='Execute file path' value={element.exePath} />
    <OtherInfomationSection
      label='Shortcut file path'
      value={element.lnkPath}
    />
    <OtherInfomationSection label='Icon file path' value={element.icon} />
    {#await gameCache then c}
      <OtherInfomationSection label='Thumbnail URL' value={c?.thumbnailUrl} />
    {/await}
  </div>
</Modal>
