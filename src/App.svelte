<script lang="ts">
  import Router from "svelte-spa-router";
  import Layout from "@/layouts/Layout.svelte";
  import { routes } from "@/router/route";
  import { routeLoaded } from "@/store/tabs";
  import { registerCollectionElementDetails } from "@/lib/registerCollectionElementDetails";

  $: setDetailPromise = registerCollectionElementDetails();
</script>

<main class="h-full w-full bg-(bg-primary) font-sans">
  {#await setDetailPromise then _}
    <Layout>
      <Router {routes} on:routeLoaded={routeLoaded} />
    </Layout>
  {/await}
</main>
