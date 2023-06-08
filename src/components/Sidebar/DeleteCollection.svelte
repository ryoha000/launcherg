<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import { commandDeleteCollection } from "@/lib/command";
  import type { Collection } from "@/lib/types";
  import { collections } from "@/store/collections";

  export let isOpen: boolean;
  export let collection: Collection | null;

  const deleteCollection = async () => {
    if (!collection) {
      return;
    }
    await commandDeleteCollection(collection.id);
    isOpen = false;
    await collections.init();
  };
</script>

{#if collection}
  <Modal
    {isOpen}
    on:close={() => (isOpen = false)}
    title={`Delete collection`}
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
          このコレクションを削除します
        </div>
        <div class="text-(body2 text-primary)">
          あなたは同名のコレクションを作り直すことはできますが、その場合このコレクションに現在含まれるゲームは引き継がれません
        </div>
      </div>
    </div>
    <div class="p-4" slot="footer">
      <Button
        text="{collection.name} を削除する"
        variant="error"
        appendClass="w-full justify-center"
        on:click={deleteCollection}
      />
    </div>
  </Modal>
{/if}
