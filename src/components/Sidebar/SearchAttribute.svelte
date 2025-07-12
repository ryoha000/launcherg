<script lang='ts'>
  import { createBubbler } from 'svelte/legacy'

  const bubble = createBubbler()
  import type { Attribute } from '@/components/Sidebar/searchAttributes'
  import {

    ATTRIBUTE_LABELS,
  } from '@/components/Sidebar/searchAttributes'

  interface Props {
    attribute: Attribute
  }

  const { attribute }: Props = $props()
</script>

<button
  class='rounded-2xl border-(border-primary solid ~) p-(x-4 y-1) hover:(border-border-button-hover) transition-all'
  class:bg-transparent={!attribute.enabled}
  class:hover:bg-bg-button-hover={!attribute.enabled}
  class:bg-text-primary={attribute.enabled}
  class:hover:bg-text-secondary={attribute.enabled}
  onclick={bubble('click')}
>
  <div
    class='text-(body3) whitespace-nowrap font-medium'
    class:text-text-primary={!attribute.enabled}
    class:text-bg-primary={attribute.enabled}
  >
    {ATTRIBUTE_LABELS[attribute.key]}
  </div>
</button>
