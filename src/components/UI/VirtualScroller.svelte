<!-- @migration-task Error while migrating Svelte code: This migration would change the name of a slot (header to header_1) making the component unusable -->
<script lang='ts'>
  import { useVirtualScroller } from '@/components/UI/virtualScroller'

  export let className = ''
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
</script>

<div use:container class='w-full h-full overflow-y-auto {className}'>
  <div use:header>
    <slot name='header' />
  </div>
  <div
    use:contents
    class='relative transform-gpu backface-hidden'
    style='height: {$virtualHeight}px;'
  >
    <slot
      {setVirtualHeight}
      {contentsWidth}
      {contentsScrollY}
      {containerHeight}
      {contentsScrollTo}
    />
  </div>
</div>
