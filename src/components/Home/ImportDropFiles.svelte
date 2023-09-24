<script lang="ts">
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import ImportManually from "@/components/Sidebar/ImportManually.svelte";
  import { commandUpsertCollectionElement } from "@/lib/command";
  import { showErrorToast, showInfoToast } from "@/lib/toast";
  import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
  import type { AllGameCacheOne } from "@/lib/types";

  onMount(() =>
    listen<string[]>("tauri://file-drop", (event) => {
      importFileDropPaths = [];
      const files = event.payload;
      for (const file of files) {
        const exts = ["exe", "lnk", "url"];
        let isIgnore = true;
        for (const ext of exts) {
          if (file.toLowerCase().endsWith(ext)) {
            isIgnore = false;
          }
        }
        if (isIgnore) {
          showErrorToast(
            "EXEファイルかショートカットファイルをドラッグアンドドロップしてください。フォルダから追加したい場合はサイドバーの Add ボタンから「自動でフォルダから追加」を選択してください。"
          );
          continue;
        }
        importFileDropPaths.push(file);
      }
      if (importFileDropPaths.length !== 0) {
        importFileDropPathIndex = 0;
        isOpenImportFileDrop = true;
      }
    })
  );

  let isOpenImportFileDrop = false;
  let importFileDropPathIndex = -1;
  let importFileDropPaths: string[] = [];

  const next = () => {
    if (importFileDropPathIndex < importFileDropPaths.length - 1) {
      isOpenImportFileDrop = true;
      importFileDropPathIndex += 1;
    } else {
      importFileDropPathIndex = -1;
    }
  };
  const importManually = async (arg: {
    exePath: string | null;
    lnkPath: string | null;
    gameCache: AllGameCacheOne;
  }) => {
    await commandUpsertCollectionElement(arg);
    await registerCollectionElementDetails();
    await sidebarCollectionElements.refetch();
    showInfoToast(`${arg.gameCache.gamename}を登録しました。`);
    isOpenImportFileDrop = false;
    setTimeout(next, 0);
  };
  const skipImport = () => {
    isOpenImportFileDrop = false;
    setTimeout(next, 0);
  };
</script>

{#if isOpenImportFileDrop && importFileDropPathIndex !== -1 && importFileDropPaths.length}
  <ImportManually
    bind:isOpen={isOpenImportFileDrop}
    path={importFileDropPaths[importFileDropPathIndex]}
    cancelText="Skip"
    on:confirm={(e) => importManually(e.detail)}
    on:cancel={skipImport}
  />
{/if}
