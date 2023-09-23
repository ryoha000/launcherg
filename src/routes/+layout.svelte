<script lang="ts">
  import "virtual:uno.css";
  import "@unocss/reset/tailwind-compat.css";
  import "./index.scss";
  import "tippy.js/dist/tippy.css";
  import "simplebar/dist/simplebar.css";
  import "easymde/dist/easymde.min.css";
  import "./toast.scss";

  import Layout from "@/layouts/Layout.svelte";
  import { initialize, onBeforeNavigate } from "@/store/tabs";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { onMount } from "svelte";
  import { initializeAllGameCache } from "@/lib/scrapeAllGame";
  import ImportDropFiles from "@/components/Home/ImportDropFiles.svelte";
  import { afterNavigate, beforeNavigate } from "$app/navigation";

  $: setDetailPromise = registerCollectionElementDetails();

  onMount(() => {
    initialize();
    initializeAllGameCache();
  });
  afterNavigate(onBeforeNavigate);
</script>

<main class="h-full w-full bg-(bg-primary) font-sans">
  {#await setDetailPromise then _}
    <Layout>
      <slot />
    </Layout>
  {/await}
  <ImportDropFiles />
</main>
