<script lang="ts">
  import Input from "@/components/UI/Input.svelte";
  import Modal from "@/components/UI/Modal.svelte";

  export let isOpen: boolean;
  let name = "";
</script>

<Modal bind:isOpen title="Add new collection" confirmText="Add">
  <div>
    <Input bind:value={name} label="Name" />
  </div>
</Modal>
