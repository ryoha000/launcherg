<svelte:options />

<script lang='ts'>
  import SimpleBar from 'simplebar'
  import { createEventDispatcher } from 'svelte'

  const dispatcher = createEventDispatcher<{ scroll: { event: Event } }>()

  let isHover = $state(false)

  const simplebar = (node: HTMLElement) => {
    const simplebar = new SimpleBar(node, {
      scrollbarMinSize: 64,
    })

    const onScroll = (e: Event) => {
      dispatcher('scroll', { event: e })
    }
    simplebar.getScrollElement()?.addEventListener('scroll', onScroll)

    const onWheel = (e: WheelEvent) => {
      if (isHover) {
        simplebar
          .getScrollElement()
          ?.scrollBy({ left: e.deltaY * 5, behavior: 'smooth' })
      }
    }
    window.addEventListener('wheel', onWheel)

    const element = simplebar.getScrollElement()
    if (element) {
      scrollBy = (options?: ScrollToOptions | undefined) => {
        element.scrollBy(options)
      }
    }
    return {
      destroy: () => {
        removeEventListener('wheel', onWheel)
        simplebar.getScrollElement()?.removeEventListener('scroll', onScroll)
        scrollBy = () => undefined
      },
    }
  }

  let { scrollBy = $bindable((options?: ScrollToOptions | undefined): void => {
    console.warn('scrollBy is not initialized')
  }), children } = $props()

  export {
    scrollBy,
  }
</script>

<div class='w-full min-w-0'>
  <div
    use:simplebar
    class='overflow-x-auto scroll-smooth'
    onmouseenter={() => (isHover = true)}
    onmouseleave={() => (isHover = false)}
  >
    {@render children?.()}
  </div>
</div>
