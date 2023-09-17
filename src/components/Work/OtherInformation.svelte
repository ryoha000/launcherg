<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import OtherInfomationSection from "@/components/Work/OtherInfomationSection.svelte";
  import {
    commandDeleteCollectionElement,
    commandGetGameCacheById,
  } from "@/lib/command";
  import type { CollectionElement } from "@/lib/types";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { deleteTab, tabs, selected } from "@/store/tabs";

  export let isOpen: boolean;
  export let element: CollectionElement;

  $: gameCache = commandGetGameCacheById(element.id);
</script>

<Modal
  {isOpen}
  on:close={() => (isOpen = false)}
  on:cancel={() => (isOpen = false)}
  title={`Infomation`}
  autofocusCloseButton
  withFooter={false}
>
  <div class="space-y-4">
    <OtherInfomationSection label="ErogameScape ID" value={element.id} />
    <OtherInfomationSection label="Execute file path" value={element.exePath} />
    <OtherInfomationSection
      label="Shortcut file path"
      value={element.lnkPath}
    />
    <OtherInfomationSection label="Icon file path" value={element.icon} />
    {#await gameCache then c}
      <OtherInfomationSection label="Thumbnail URL" value={c?.thumbnailUrl} />
    {/await}
  </div>
</Modal>
