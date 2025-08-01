<script lang='ts'>
  import type { Attribute } from '@/components/Sidebar/searchAttributes'
  import {

    ATTRIBUTE_LABELS,
  } from '@/components/Sidebar/searchAttributes'

  interface Props {
    attribute: Attribute
    onclick?: (e: Event) => void
  }

  const { attribute, onclick }: Props = $props()
</script>

<button
  class='border border-(border-primary solid) rounded-2xl p-(x-4 y-1) transition-all hover:(border-border-button-hover)'
  class:bg-transparent={!attribute.enabled}
  class:hover:bg-bg-button-hover={!attribute.enabled}
  class:bg-text-primary={attribute.enabled}
  class:hover:bg-text-secondary={attribute.enabled}
  {onclick}
>
  <div
    class='whitespace-nowrap text-(body3) font-medium'
    class:text-text-primary={!attribute.enabled}
    class:text-bg-primary={attribute.enabled}
  >
    {ATTRIBUTE_LABELS[attribute.key]}
  </div>
</button>
