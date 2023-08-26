<script lang="ts">
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import { type Props as TippyOption } from "tippy.js";

  export let leftIcon = "";
  export let rightIcon = "";
  export let appendClass = "";
  export let text = "";
  export let type: "button" | "submit" | undefined = undefined;
  export let tooltip: Partial<TippyOption> | undefined = undefined;
  export let variant: Variant = "normal";
  export let disabled = false;
  export let wrappable = false;

  let iconSizeClass = "w-4 h-4";

  let iconVarinatClass = "color-ui-tertiary";
  $: {
    if (variant === "success") {
      iconVarinatClass = "color-text-white";
    }
  }
</script>

<ButtonBase
  appendClass={`${appendClass} ${
    wrappable ? "" : "h-8"
  } px-3 gap-2 flex items-center`}
  {variant}
  {type}
  {tooltip}
  {disabled}
  on:click
>
  {#if leftIcon}
    <div class={`${iconVarinatClass} ${iconSizeClass} ${leftIcon}`} />
  {/if}
  {#if text}
    <div
      class={`text-body2 font-medium ${wrappable ? "" : "whitespace-nowrap"}`}
    >
      {text}
    </div>
  {/if}
  {#if rightIcon}
    <div class={`${iconVarinatClass} ${iconSizeClass} ${rightIcon}`} />
  {/if}
</ButtonBase>
