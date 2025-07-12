<!-- @migration-task Error while migrating Svelte code: This migration would change the name of a slot (header to header_1) making the component unusable -->
<script lang='ts'>
  import { useVirtualScroller } from '@/components/UI/virtualScroller'

  const {
    container,
    header,
    contents,
    virtualHeight,
    setVirtualHeight,
    contentsWidth,
    contentsScrollY,
    containerHeight,
    contentsScrollTo,
  } = useVirtualScroller()

  const { className, topElement, children } = $props<{
    className?: string
    topElement?: import('svelte').Snippet<[]>
    children?: import('svelte').Snippet<[{
      setVirtualHeight: typeof setVirtualHeight
      contentsWidth: typeof contentsWidth
      contentsScrollY: typeof contentsScrollY
      containerHeight: typeof containerHeight
      contentsScrollTo: typeof contentsScrollTo
    }]>
  }>()
</script>

<div use:container class='w-full h-full overflow-y-auto {className}'>
  <div use:header>
    {@render topElement?.({})}
  </div>
  <div
    use:contents
    class='relative transform-gpu backface-hidden'
    style='height: {$virtualHeight}px;'
  >
    {@render children?.({
      setVirtualHeight,
      contentsWidth,
      contentsScrollY,
      containerHeight,
      contentsScrollTo,
    })}
  </div>
</div>
