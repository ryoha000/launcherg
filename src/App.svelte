<script lang="ts">
  import Router from "svelte-spa-router";
  import Layout from "@/layouts/Layout.svelte";
  import { routes } from "@/router/route";
  import { routeLoaded } from "@/store/tabs";
</script>

<main class="h-full w-full bg-(bg-primary) font-sans">
  <Layout>
    <Router {routes} on:routeLoaded={routeLoaded} />
  </Layout>
</main>
