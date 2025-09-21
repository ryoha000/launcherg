<script lang='ts'>
  import { Router } from '@mateothegreat/svelte5-router'
  import { QueryClientProvider } from '@tanstack/svelte-query'
  import { onMount } from 'svelte'
  import ImportDropFiles from '@/components/Home/ImportDropFiles.svelte'
  import Titlebar from '@/components/UI/Titlebar/Titlebar.svelte'
  import Layout from '@/layouts/Layout.svelte'
  import { queryClient } from '@/lib/data/queryClient'
  import { useEvent } from '@/lib/event'
  import { registerCollectionElementDetails } from '@/lib/registerCollectionElementDetails'
  import { initializeAllGameCache } from '@/lib/scrape/scrapeAllGame'
  import { showErrorToast, showInfoToast } from '@/lib/toast'
  import { routes } from '@/router/route'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'
  import { initialize, routeLoaded } from '@/store/tabs'

  const appEvent = useEvent()
  const setDetailPromise = $derived(registerCollectionElementDetails())

  onMount(() => {
    initialize()
    initializeAllGameCache()

    void appEvent.startListen('appSignal:showMessage', ({ event }) => {
      if (event.type !== 'showMessage')
        return

      showInfoToast(event.payload.message)
    })

    void appEvent.startListen('appSignal:showErrorMessage', ({ event }) => {
      if (event.type !== 'showErrorMessage')
        return

      showErrorToast(event.payload.message)
    })

    void appEvent.startListen('appSignal:refetchWorks', ({ event }) => {
      if (event.type !== 'refetchWorks')
        return

      void sidebarCollectionElements.refetch()
    })

    return () => {
      appEvent.stopAll()
    }
  })

</script>

<main class='grid grid-rows-[auto_1fr] h-full w-full bg-(bg-primary) font-sans'>
  <Titlebar />
  {#await setDetailPromise then _}
    <QueryClientProvider client={queryClient}>
      <Layout>
        <Router {routes} hooks={{ post: routeLoaded }} />
      </Layout>
    </QueryClientProvider>
  {/await}
  <ImportDropFiles />
</main>
