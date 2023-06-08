<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import Checkbox from "@/components/UI/Checkbox.svelte";
  import Input from "@/components/UI/Input.svelte";
  import Modal from "@/components/UI/Modal.svelte";
  import {
    commandAddCollectionElementsInPc,
    commandGetDefaultImportDirs,
  } from "@/lib/command";
  import { showInfoToast } from "@/lib/toast";
  import type { Collection } from "@/lib/types";
  import { createLocalStorageWritable } from "@/lib/utils";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import { onMount } from "svelte";

  export let isOpen: boolean;
  export let collection: Collection | null = null;

  let inputContainer: HTMLDivElement | null = null;

  let useCache = true;
  const [paths, getPaths] = createLocalStorageWritable<
    { id: number; path: string }[]
  >("auto-import-dir-paths", [
    { id: Math.floor(Math.random() * 100000), path: "" },
  ]);
  const updatePath = (index: number, value: string) => {
    paths.update((v) => {
      v[index].path = value;
      return v;
    });
  };
  const removePath = (index: number) => {
    paths.update((v) => {
      v = [...v.slice(0, index), ...v.slice(index + 1)];
      return v;
    });
  };
  const addEmptyPath = async () => {
    if (
      getPaths().length > 0 &&
      getPaths()[getPaths().length - 1].path === ""
    ) {
      return;
    }
    paths.update((v) => {
      v.push({ id: new Date().getTime(), path: "" });
      return v;
    });
    await new Promise((resolve) => setTimeout(resolve, 0));
    if (inputContainer) {
      const inputs = inputContainer.getElementsByTagName("input");
      if (inputs.length > 0) {
        inputs[inputs.length - 1].focus();
      }
    }
  };
  const confirm = async () => {
    const res = await commandAddCollectionElementsInPc(
      getPaths().map((v) => v.path),
      useCache,
      collection?.id || null
    );
    if (collection) {
      await sidebarCollectionElements.init(collection.id);
    } else {
      await sidebarCollectionElements.refetch();
    }

    const text = res.length
      ? `「${res[0]}」${
          res.length === 1 ? "が" : `、他${res.length}件`
        }追加されました`
      : "新しく追加されたゲームはありません";

    showInfoToast(text);
    isOpen = false;
  };

  onMount(async () => {
    const defaultPaths = await commandGetDefaultImportDirs();
    paths.update((v) => {
      const appendPaths = [];
      for (const defaultPath of defaultPaths) {
        if (!v.some((v) => v.path === defaultPath)) {
          appendPaths.push({
            id: Math.floor(Math.random() * 100000),
            path: defaultPath,
          });
        }
      }
      return [...appendPaths, ...v];
    });
  });
</script>

{#if collection}
  <Modal
    bind:isOpen
    title="Automatically import game"
    confirmText="Start import"
    fullmodal
    on:confirm={confirm}
  >
    <div class="space-y-8">
      <div class="space-y-4">
        <div class="text-(text-primary h4) font-medium">
          自動追加するフォルダ
        </div>
        <form
          class="flex flex-col gap-2"
          on:submit|preventDefault={addEmptyPath}
        >
          {#each $paths as path, i (path.id)}
            <div class="flex items-center gap-8" bind:this={inputContainer}>
              <div class="flex-1">
                <Input
                  value={path.path}
                  on:update={(e) => updatePath(i, e.detail.value)}
                />
              </div>
              <button
                on:click={() => removePath(i)}
                type="button"
                tabindex={-1}
                class="ml-auto w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all"
              />
            </div>
          {/each}
          <Button
            appendClass="m-auto"
            leftIcon="i-iconoir-plus"
            text="Add folder path"
            type="submit"
            on:click={addEmptyPath}
          />
        </form>
      </div>
      <div class="space-y-2">
        <div class="text-(text-primary h4) font-medium">オプション</div>
        <!-- svelte-ignore a11y-label-has-associated-control -->
        <label class="flex gap-2 cursor-pointer">
          <Checkbox bind:value={useCache} />
          <div>
            <div class="text-(text-primary body) font-medium">
              前回から追加されたファイルのみを追加する
            </div>
            <div class="text-(text-tertiary body2)">
              自動追加が初回の場合このオプションは意味を持ちません。このオプションがオフの場合、自動追加は2分程度かかる場合があります。
            </div>
          </div>
        </label>
      </div>
    </div>
  </Modal>
{/if}
