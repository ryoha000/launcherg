<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Input from "@/components/UI/Input.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { createEventDispatcher } from "svelte";

  export let path: string;
  export let label: string;
  export let placeholder: string = "";
  export let withFilter = true;
  export let directory = false;

  const dispatcher = createEventDispatcher<{ update: { value: string } }>();

  const openDialog = async () => {
    const selected = await open({
      multiple: false,
      filters: withFilter
        ? [
            {
              name: "exe",
              extensions: ["exe", "EXE", "lnk"],
            },
          ]
        : [],
      directory,
    });
    if (selected === null || Array.isArray(selected)) {
      return;
    }
    path = selected;
    dispatcher("update", { value: selected });
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
