<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Input from "@/components/UI/Input.svelte";
  import { open } from "@tauri-apps/api/dialog";

  export let path: string;
  export let label: string;
  export let placeholder: string = "";

  const openDialog = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "exe",
          extensions: ["exe", "EXE"],
        },
      ],
    });
    if (selected === null || Array.isArray(selected)) {
      return;
    }
    path = selected;
  };
</script>

<div class="flex gap-2 items-end">
  <div class="flex-1">
    <Input value={path} {label} {placeholder} on:update />
  </div>
  <Button
    leftIcon="i-material-symbols-folder-outline-rounded"
    on:click={openDialog}
  />
</div>
