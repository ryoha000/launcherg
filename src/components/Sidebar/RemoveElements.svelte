<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import {
    commandDeleteCollection,
    commandRemoveElementsFromCollection,
  } from "@/lib/command";
  import { showInfoToast } from "@/lib/toast";
  import type { Collection, CollectionElement } from "@/lib/types";
  import { collections } from "@/store/collections";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";

  export let isOpen: boolean;
  export let collection: Collection | null;
  export let removeElements: CollectionElement[];

  $: removeElementsText = `「${
    collection?.name
  }」から削除されるのは、${removeElements
    .filter((v, i) => i < 5)
    .map((v) => `「${v.gamename}」`)
    .join("、")}${
    removeElements.length > 5 ? `他${removeElements.length - 5}件` : ""
  }です。`;

  const remove = async () => {
    if (!collection) {
      return;
    }
    await commandRemoveElementsFromCollection(
      collection.id,
      removeElements.map((v) => v.id)
    );
    await sidebarCollectionElements.init(collection.id);
    showInfoToast("コレクションからの削除が完了しました");
    isOpen = false;
  };
</script>

{#if collection && removeElements.length}
  <Modal
    bind:isOpen
    title={`Remove games from collection`}
    withContentPadding={false}
    autofocusCloseButton
    maxWidth="max-w-110"
    headerClass="border-b-(border-warning opacity-40) "
  >
    <div
      class="bg-bg-warning border-(b-1px solid border-warning opacity-40) flex gap-2 p-(x-4 y-5)"
    >
      <div
        class="w-6 h-6 i-material-symbols-warning-outline-rounded color-accent-warning"
      >
        warning icon
      </div>
      <div class="space-y-1">
        <div class="text-(body text-primary) font-medium">
          選択したゲームをコレクションから削除します
        </div>
        <div class="text-(body2 text-primary)">
          {removeElementsText}
        </div>
      </div>
    </div>
    <div class="p-4" slot="footer">
      <Button
        text="コレクションから削除する"
        variant="error"
        appendClass="w-full justify-center"
        on:click={remove}
      />
    </div>
  </Modal>
{/if}
