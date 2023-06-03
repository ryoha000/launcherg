<script lang="ts">
  import Router from "svelte-spa-router";
  import Layout from "@/layouts/Layout.svelte";
  import { routes } from "@/router/route";
</script>

<main class="h-full w-full bg-(bg-primary) font-sans">
  <Layout>
    <Router {routes} />
  </Layout>
</main>
