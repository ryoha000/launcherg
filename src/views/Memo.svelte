<script lang="ts">
  import EasyMDE from "easymde";
  import { readImage } from "tauri-plugin-clipboard-api";
  import {
    commandSaveScreenshotByPid,
    commandUploadImage,
  } from "@/lib/command";
  import { convertFileSrc } from "@tauri-apps/api/tauri";
  import { open } from "@tauri-apps/api/dialog";
  import { memo } from "@/store/memo";
  import { skyWay } from "@/store/skyway";
  import { startProcessMap } from "@/store/startProcessMap";

  export let params: { id: string };
  $: id = +params.id;

  let height: number;

  const mde = (node: HTMLElement) => {
    const easyMDE = new EasyMDE({
      element: node,
      spellChecker: false,
      sideBySideFullscreen: false,
      previewImagesInEditor: true,
      autofocus: true,
      autosave: {
        enabled: true,
        delay: 1000,
        uniqueId: `memo-${id}`,
      },
      toolbar: [
        "bold",
        "italic",
        "heading",
        "|",
        "quote",
        "unordered-list",
        "ordered-list",
        "|",
        "link",
        {
          name: "image",
          action: async () => {
            const selected = await open({
              multiple: false,
              filters: [
                {
                  name: "Image",
                  extensions: ["png", "jpeg", "jpg", "*"],
                },
              ],
            });
            if (selected === null || Array.isArray(selected)) {
              return;
            }
            insertImage(selected);
          },
          className: "fa fa-picture-o",
          title: "Insert image",
        },
        {
          name: "screenshot",
          action: async () => {
            const startProcessId = $startProcessMap[id];
            console.log({ startProcessId });
            const screenshotPath = await commandSaveScreenshotByPid(
              id,
              startProcessId
            );
            insertImage(screenshotPath);
          },
          className: "fa fa-desktop",
          title: "Insert screenshot",
        },
      ],
      imagesPreviewHandler: (imagePath) => convertFileSrc(imagePath),
    });
    const onPaste = async () => {
      try {
        const base64Image = await readImage();
        const imagePath = await commandUploadImage(id, base64Image);
        insertImage(imagePath);
      } catch {}
    };
    const insertImage = (imagePath: string) => {
      const cursor = easyMDE.codemirror.getCursor();
      const prev = easyMDE.value();
      const lines = prev.split("\n");
      const newLines: string[] = [];
      for (let i = 0; i < lines.length; i++) {
        newLines.push(lines[i]);
        if (i === cursor.line) {
          newLines.push(`![](${imagePath})`);
          newLines.push("");
        }
      }
      easyMDE.codemirror.setValue(newLines.join("\n"));
      easyMDE.codemirror.setCursor({ line: cursor.line + 2, ch: 0 });
    };
    const ele = document.querySelector(".EasyMDEContainer");
    ele?.addEventListener("paste", onPaste);

    const syncTimer = setInterval(() => {
      const current = easyMDE.value();
      if ($memo.find((v) => v.workId === id)?.value !== current) {
        memo.update((memos) =>
          memos.reduce(
            (acc, cur) => {
              if (cur.workId !== id) acc.push(cur);
              return acc;
            },
            [
              { workId: id, value: current, lastModified: "local" },
            ] as typeof $memo
          )
        );
        skyWay.syncMemo(id, current);
      }
    }, 1000);

    const unsubscribe = memo.subscribe((memos) => {
      const memo = memos.find((v) => v.workId === id);
      if (memo?.lastModified === "remote" && easyMDE.value() !== memo.value) {
        easyMDE.value(memo.value);
      }
    });
    return {
      destroy: () => {
        ele?.removeEventListener("paste", onPaste);
        unsubscribe();
        clearInterval(syncTimer);
      },
    };
  };
</script>

<div class="w-full h-full min-w-0" bind:clientHeight={height}>
  <textarea id="mde" use:mde />
</div>
