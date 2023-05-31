<script lang="ts">
  import tippy, { type Props as TippyOption } from "tippy.js";
  export let appendClass = "";
  export let tooltip: Partial<TippyOption> | undefined = undefined;

  const tooltipAction = (node: HTMLElement) => {
    if (!tooltip) {
      return;
    }

    const tp = tippy(node, tooltip);

    return {
      update() {
        if (!tooltip) {
          return;
        }
        tp.setProps(tooltip);
      },
      destroy() {
        tp.destroy();
      },
    };
  };
</script>

<button
  use:tooltipAction
  class={`border-(~ border-button opacity-10 solid) rounded bg-bg-button transition-all hover:(border-border-button-hover bg-bg-button-hover) ${appendClass}`}
  on:click
>
  <slot />
</button>
