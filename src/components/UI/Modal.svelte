<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import ModalBase from "@/components/UI/ModalBase.svelte";
  import { createEventDispatcher } from "svelte";

  export let isOpen = false;
  export let autofocusCloseButton = false;
  export let maxWidth = "";
  export let headerClass = "";
  export let title = "";
  export let confirmText = "";
  export let withFooter = true;
  export let withContentPadding = true;
  export let fullmodal = false;
  export let confirmDisabled = false;

  const dispatcher = createEventDispatcher<{
    confirm: {};
    cancel: {};
    close: {};
  }>();
</script>

<ModalBase
  {isOpen}
  panelClass={maxWidth ? maxWidth : "max-w-160"}
  {fullmodal}
  on:close
>
  <div class="grid grid-rows-[min-content_1fr_min-content] h-full">
    <div
      class="flex items-center bg-bg-secondary border-(b-1px solid border-primary) {headerClass}"
    >
      <div class="px-4 text-(text-primary body) font-medium">
        {title}
      </div>
      <button
        on:click={() => dispatcher("close")}
        class="ml-auto p-4 bg-transparent color-text-tertiary hover:color-text-primary transition-all"
        tabindex={autofocusCloseButton ? 0 : -1}
      >
        <div class="w-5 h-5 i-iconoir-cancel" />
      </button>
    </div>
    <div class:p-4={withContentPadding} class="overflow-y-auto">
      <slot />
    </div>
    {#if withFooter}
      <slot name="footer">
        <div class="flex items-center p-4 border-(t-1px solid border-primary)">
          <div class="flex items-center ml-auto gap-2">
            <Button text="Cancel" on:click={() => dispatcher("close")} />
            <Button
              variant="success"
              disabled={confirmDisabled}
              text={confirmText}
              on:click={() => dispatcher("confirm")}
            />
          </div>
        </div>
      </slot>
    {/if}
  </div>
</ModalBase>
