<script lang="ts">
  import {
    ATTRIBUTE_LABELS,
    type Attribute,
  } from "@/components/Sidebar/searchAttributes";

  export let attribute: Attribute;
</script>

<button
  class="rounded-2xl border-(border-primary solid ~) p-(x-4 y-1) hover:(border-border-button-hover) transition-all"
  class:bg-transparent={!attribute.enabled}
  class:hover:bg-bg-button-hover={!attribute.enabled}
  class:bg-text-primary={attribute.enabled}
  class:hover:bg-text-secondary={attribute.enabled}
  on:click
>
  <div
    class="text-(body3) whitespace-nowrap font-medium"
    class:text-text-primary={!attribute.enabled}
    class:text-bg-primary={attribute.enabled}
  >
    {ATTRIBUTE_LABELS[attribute.key]}
  </div>
</button>
