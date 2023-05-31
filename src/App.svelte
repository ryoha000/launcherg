<script lang="ts">
  import { Router, Link, Route } from "svelte-routing";
  import Layout from "@/layouts/Layout.svelte";

  export let url = "/";
</script>

<main class="h-full w-full bg-(bg-primary) font-sans">
  <Router {url}>
    <Layout>
      <div class="text-(h1 text-primary)">hoge</div>
    </Layout>
  </Router>
</main>
