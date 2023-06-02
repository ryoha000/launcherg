<script lang="ts">
  import { Router, Link, Route } from "svelte-routing";
  import Layout from "@/layouts/Layout.svelte";
  import Button from "@/components/UI/Button.svelte";
  import { addCollectionElementsInPc } from "@/lib/command";

  export let url = "/";

  const exp = async () => {
    console.log(
      await addCollectionElementsInPc(
        ["C:\\Users\\ryoha\\Desktop\\既プレイゲーム"],
        false,
        null
      )
    );
  };
</script>

<main class="h-full w-full bg-(bg-primary) font-sans">
  <Router {url}>
    <Layout>
      <div>
        <div class="text-(h1 text-primary)">hoge</div>
        <Button text="explore button" on:click={exp} />
      </div>
    </Layout>
  </Router>
</main>
