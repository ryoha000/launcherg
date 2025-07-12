import { derived, get, writable } from 'svelte/store'

export function useVirtualScroller() {
  const virtualHeight = writable(0)
  const setVirtualHeight = (value: number) => virtualHeight.set(value)

  const scrollY = writable(0)
  const containerHeight = writable(0)
  let containerNode: HTMLElement | null = null

  const headerHeight = writable(0)

  const contentsWidth = writable(0)

  const contentsScrollY = derived<
    [typeof scrollY, typeof headerHeight],
    number
  >([scrollY, headerHeight], ([$scrollY, $headerHeight], set) => {
    set(Math.max(0, $scrollY - $headerHeight))
  })

  let notAppliedContentsScrollY = 0
  const contentsScrollTo = (y: number) => {
    const to = y + get(headerHeight)
    if (!containerNode || containerNode.scrollHeight < to) {
      notAppliedContentsScrollY = y
      return
    }
    containerNode.scrollTo({ top: to })
    notAppliedContentsScrollY = 0
  }

  virtualHeight.subscribe(() => {
    if (notAppliedContentsScrollY) {
      contentsScrollTo(notAppliedContentsScrollY)
    }
  })

  const header = (node: HTMLElement) => {
    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (!entry) {
        return
      }
      headerHeight.set(entry.contentRect.height)
    })
    resizeObserver.observe(node)

    return {
      destroy() {
        resizeObserver.disconnect()
      },
    }
  }
  const contents = (node: HTMLElement) => {
    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (!entry) {
        return
      }
      contentsWidth.set(entry.contentRect.width)
    })
    resizeObserver.observe(node)

    return {
      destroy() {
        resizeObserver.disconnect()
      },
    }
  }

  const container = (node: HTMLElement) => {
    containerNode = node
    if (notAppliedContentsScrollY) {
      contentsScrollTo(notAppliedContentsScrollY)
    }
    const onScroll = (e: Event) => {
      const target = e.target
      if (!(target instanceof HTMLElement)) {
        return
      }
      scrollY.set(target.scrollTop)
    }
    node.addEventListener('scroll', onScroll)

    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (!entry) {
        return
      }
      containerHeight.set(entry.contentRect.height)
    })
    resizeObserver.observe(node)

    return {
      destroy() {
        containerNode = null
        node.removeEventListener('scroll', onScroll)
        resizeObserver.disconnect()
      },
    }
  }

  return {
    container,
    header,
    contents,
    virtualHeight,
    setVirtualHeight,
    contentsWidth,
    contentsScrollY,
    containerHeight,
    contentsScrollTo,
  }
}
