<script lang="ts">
  import Router from "svelte-spa-router";
  import Layout from "@/layouts/Layout.svelte";
  import { routes } from "@/router/route";
  import { initialize, routeLoaded } from "@/store/tabs";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";
  import { onMount } from "svelte";
  import { initializeAllGameCache } from "@/lib/scrapeAllGame";
  import ImportDropFiles from "@/components/Home/ImportDropFiles.svelte";

  $: setDetailPromise = registerCollectionElementDetails();

  onMount(() => {
    initialize();
    initializeAllGameCache();
  });
</script>

<main class="h-full w-full bg-(bg-primary) font-sans">
  {#await setDetailPromise then _}
    <Layout>
      <Router {routes} on:routeLoaded={routeLoaded} />
    </Layout>
  {/await}
  <ImportDropFiles />
</main>
