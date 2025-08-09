<svelte:options />

<script lang='ts'>
  import SimpleBar from 'simplebar'

  let isHover = $state(false)

  let { onscroll, children } = $props<{
    onscroll?: (e: Event) => void
    children?: import('svelte').Snippet<[]>
  }>()

  let scrollByImplementation: (options?: ScrollToOptions) => void = () => {
    console.warn('scrollBy is not implemented')
  }
  export function scrollBy(options?: ScrollToOptions) {
    scrollByImplementation(options)
  }

  const simplebar = (node: HTMLElement) => {
    const simplebar = new SimpleBar(node, {
      scrollbarMinSize: 64,
    })

    simplebar.getScrollElement()?.addEventListener('scroll', onscroll)

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
      scrollByImplementation = (options?: ScrollToOptions) => {
        element.scrollBy(options)
      }
    }
    return {
      destroy: () => {
        removeEventListener('wheel', onWheel)
        simplebar.getScrollElement()?.removeEventListener('scroll', onscroll)
        scrollByImplementation = () => undefined
      },
    }
  }
</script>

<div class='min-w-0 w-full'>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    use:simplebar
    class='overflow-x-auto scroll-smooth'
    onmouseenter={() => (isHover = true)}
    onmouseleave={() => (isHover = false)}
  >
    {@render children?.()}
  </div>
</div>
