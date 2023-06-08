<script lang="ts">
  import Input from "@/components/UI/Input.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import { commandUpdateCollection } from "@/lib/command";
  import type { Collection } from "@/lib/types";
  import { collections } from "@/store/collections";

  export let isOpen: boolean;
  export let collection: Collection | null;
  let name = "";

  const update = async () => {
    if (!collection) {
      return;
    }
    await commandUpdateCollection(collection.id, name);
    await collections.init();
    isOpen = false;
  };
</script>

{#if collection}
  <Modal
    {isOpen}
    on:close={() => (isOpen = false)}
    title={`Change "${collection.name}"`}
    confirmText="Change"
    confirmDisabled={!name}
    on:confirm={update}
  >
    <div>
      <Input
        value={collection.name}
        label="Name"
        on:update={(e) => (name = e.detail.value)}
      />
    </div>
  </Modal>
{/if}
