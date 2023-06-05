<script lang="ts">
  import EasyMDE from "easymde";
  // import { readText } from "@tauri-apps/api/clipboard";
  import {
    writeText,
    readText,
    readImage,
    writeImage,
  } from "tauri-plugin-clipboard-api";
  import { writeBinaryFile, BaseDirectory } from "@tauri-apps/api/fs";

  export let params: { id: string };

  let height: number;

  const mde = (node: HTMLElement) => {
    console.log(params.id);
    const easyMDE = new EasyMDE({
      element: node,
      spellChecker: false,
      sideBySideFullscreen: false,
      previewImagesInEditor: true,
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
      imageUploadFunction: uploadImage,
    });
    document.querySelector(".EasyMDEContainer")?.addEventListener(
      "paste",
      async (e) => {
        console.log("paste");
        try {
          console.log(await readText());
        } catch (e) {
          console.warn("readText", e);
        }
        try {
          const base64Image = await readImage();
          await writeBinaryFile(
            "hoge.png",
            new Uint8Array(
              atob(base64Image)
                .split("")
                .map((char) => char.charCodeAt(0))
            ),
            { dir: BaseDirectory.AppConfig }
          );
          console.log(await readImage());
        } catch (e) {
          console.warn("readImage", e);
        }
      },
      true
    );
  };
  const uploadImage = (file, onSuccess, onError) => {
    console.log(file);
  };
</script>

<div class="w-full h-full" bind:clientHeight={height}>
  <textarea id="mde" use:mde />
</div>
