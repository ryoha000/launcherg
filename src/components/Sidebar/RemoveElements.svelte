<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import type { Collection, CollectionElement } from "@/lib/types";
  import { createEventDispatcher } from "svelte";

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

  const dispather = createEventDispatcher<{ confirmRemove: {} }>();
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
        on:click={() => dispather("confirmRemove")}
      />
    </div>
  </Modal>
{/if}
