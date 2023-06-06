<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import ModalBase from "@/components/UI/ModalBase.svelte";

  export let isOpen = false;
  export let title = "";
  export let confirmText = "";
</script>

<ModalBase bind:isOpen panelClass="max-w-160">
  <div>
    <div
      class="flex items-center bg-bg-secondary border-(b-1px solid border-primary)"
    >
      <div class="px-4 text-(text-primary body) font-medium">
        {title}
      </div>
      <button
        on:click={() => (isOpen = false)}
        class="ml-auto p-4 bg-transparent color-text-tertiary hover:color-text-primary transition-all"
        tabindex={-1}
      >
        <div class="w-5 h-5 i-iconoir-cancel" />
      </button>
    </div>
    <div class="p-4">
      <slot />
    </div>
    <div class="flex items-center p-4 border-(t-1px solid border-primary)">
      <div class="flex items-center ml-auto gap-2">
        <Button text="Cancel" />
        <Button variant="success" text={confirmText} />
      </div>
    </div>
  </div>
</ModalBase>
