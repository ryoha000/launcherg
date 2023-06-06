<script lang="ts">
  import Input from "@/components/UI/Input.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import type { Collection } from "@/lib/types";
  import { collections } from "@/store/collections";

  export let isOpen: boolean;
  export let collection: Collection | null;
  let name = "";

  const update = async () => {
    // TODO: update name
    await collections.init();
    isOpen = false;
  };
</script>

{#if collection}
  <Modal
    bind:isOpen
    title={`Change "${collection.name}"`}
    confirmText="Change"
    on:confirm={update}
  >
    <div>
      <Input bind:value={name} label="Name" />
    </div>
  </Modal>
{/if}
