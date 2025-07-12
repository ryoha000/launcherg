<script lang='ts'>
  import { Router } from '@mateothegreat/svelte5-router'
  import { onMount } from 'svelte'
  import ImportDropFiles from '@/components/Home/ImportDropFiles.svelte'
  import Layout from '@/layouts/Layout.svelte'
  import { registerCollectionElementDetails } from '@/lib/registerCollectionElementDetails'
  import { initializeAllGameCache } from '@/lib/scrape/scrapeAllGame'
  import { routes } from '@/router/route'
  import { initialize, routeLoaded } from '@/store/tabs'

  const setDetailPromise = $derived(registerCollectionElementDetails())

  onMount(() => {
    initialize()
    initializeAllGameCache()
  })

</script>

<main class='h-full w-full bg-(bg-primary) font-sans'>
  {#await setDetailPromise then _}
    <Layout>
      <Router {routes} hooks={{ post: routeLoaded }} />
    </Layout>
  {/await}
  <ImportDropFiles />
</main>
