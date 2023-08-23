<script lang="ts">
  import Router from "svelte-spa-router";
  import Layout from "@/layouts/Layout.svelte";
  import { routes } from "@/router/route";
  import { initialize, routeLoaded } from "@/store/tabs";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { initializeAllGameCache } from "@/lib/scrapeAllGame";

  $: setDetailPromise = registerCollectionElementDetails();

  onMount(() => {
    initialize();
    initializeAllGameCache();
    const unlisten = listen("tauri://file-drop", (event) => {
      console.log(event);
    });
    return unlisten;
  });
</script>

<main class="h-full w-full bg-(bg-primary) font-sans">
  {#await setDetailPromise then _}
    <Layout>
      <Router {routes} on:routeLoaded={routeLoaded} />
    </Layout>
  {/await}
</main>
