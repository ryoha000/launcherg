<script lang="ts">
  import Input from "@/components/UI/Input.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import { commandCreateNewCollection } from "@/lib/command";
  import { collections } from "@/store/collections";

  export let isOpen: boolean;
  let name = "";

  const createNewCollection = async () => {
    await commandCreateNewCollection(name);
    await collections.init();
    isOpen = false;
  };
</script>

<Modal
  bind:isOpen
  title="Create new collection"
  confirmText="Create"
  on:confirm={createNewCollection}
>
  <div>
    <Input bind:value={name} label="Name" />
  </div>
</Modal>
