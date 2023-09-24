<script lang="ts">
  import tippy, { type Props as TippyOption } from "tippy.js";
  export let appendClass = "";
  export let type: "button" | "submit" | undefined = undefined;
  export let tooltip: Partial<TippyOption> | undefined = undefined;
  export let disabled = false;

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

  export let variant: Variant = "normal";

  let buttonVariantClass = "";
  $: {
    switch (variant) {
      case "normal":
        buttonVariantClass =
          "bg-bg-button border-(~ border-button opacity-10 solid) text-text-primary hover:(border-border-button-hover bg-bg-button-hover)";
        break;
      case "accent":
        buttonVariantClass =
          "bg-bg-button border-(~ border-button opacity-10 solid) text-accent-accent hover:(border-accent-accent bg-accent-accent text-text-secondary)";
        break;
      case "error":
        buttonVariantClass =
          "bg-bg-button border-(~ border-button opacity-10 solid) text-accent-error hover:(border-accent-error bg-accent-error text-text-secondary)";
        break;
      case "success":
        buttonVariantClass = disabled
          ? "bg-bg-success-disabled border-(~ solid border-success-disabled) text-text-success-disabled"
          : "bg-accent-success border-(~ solid accent-success) text-text-white hover:bg-bg-success-hover";
        break;
      default:
        const _: never = variant;
        break;
    }
  }
</script>

<button
  use:tooltipAction
  {type}
  {disabled}
  class={`rounded transition-all ${buttonVariantClass} ${appendClass}`}
  on:click
>
  <slot />
</button>
