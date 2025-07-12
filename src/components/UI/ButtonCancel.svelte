<script lang="ts">
  import { createBubbler } from 'svelte/legacy';

  const bubble = createBubbler();
  interface Props {
    icon?: string;
  }

  let { icon = "i-iconoir-cancel" }: Props = $props();
</script>

<button
  onclick={bubble('click')}
  type="button"
  tabindex={-1}
  class="ml-auto w-5 h-5 {icon} color-text-tertiary hover:color-text-primary transition-all flex-shrink-0"
></button>
