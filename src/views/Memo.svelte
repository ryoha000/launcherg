<script lang="ts">
  import EasyMDE from "easymde";
  import { readImage } from "tauri-plugin-clipboard-api";
  import { commandUploadImage } from "@/lib/command";
  import { convertFileSrc } from "@tauri-apps/api/tauri";

  export let params: { id: string };
  $: id = +params.id;

  let height: number;

  const mde = (node: HTMLElement) => {
    console.log(params.id);
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
        "image",
      ],
      imagesPreviewHandler: (imagePath) => convertFileSrc(imagePath),
    });
    const onPaste = async () => {
      try {
        const base64Image = await readImage();
        const imagePath = await commandUploadImage(id, base64Image);
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
      } catch {}
    };
    const ele = document.querySelector(".EasyMDEContainer");
    ele?.addEventListener("paste", onPaste);

    return {
      destroy: () => {
        ele?.removeEventListener("paste", onPaste);
      },
    };
  };
</script>

<div class="w-full h-full min-w-0" bind:clientHeight={height}>
  <textarea id="mde" use:mde />
</div>
