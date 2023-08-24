<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import { commandDeleteCollectionElement } from "@/lib/command";
  import type { CollectionElement } from "@/lib/types";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { deleteTab, tabs, selected } from "@/store/tabs";

  export let isOpen: boolean;
  export let element: CollectionElement;

  const deleteGame = async () => {
    await commandDeleteCollectionElement(element.id);
    await sidebarCollectionElements.refetch();
    deleteTab($tabs[$selected].id);
    isOpen = false;
  };
</script>

<Modal
  {isOpen}
  on:close={() => (isOpen = false)}
  on:cancel={() => (isOpen = false)}
  title={`Delete game`}
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
    />
    <div class="space-y-1">
      <div class="text-(body text-primary) font-medium">
        このゲームの登録を削除します
      </div>
      <div class="text-(body2 text-primary)">
        参照元のファイルが消えることはありません。プレイ時間のデータは同じゲームを登録したとき引き継がれます。
      </div>
    </div>
  </div>
  <div class="p-4" slot="footer">
    <Button
      text="{element.gamename} を削除する"
      variant="error"
      appendClass="w-full justify-center"
      on:click={deleteGame}
    />
  </div>
</Modal>
