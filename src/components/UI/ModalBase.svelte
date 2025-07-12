<script lang="ts">
  import { createDialog } from "svelte-headlessui";
  import Transition from "svelte-transition";
  import { createEventDispatcher, onMount } from "svelte";
  interface Props {
    isOpen?: boolean;
    panelClass?: string;
    fullmodal?: boolean;
    children?: import('svelte').Snippet;
  }

  let {
    isOpen = true,
    panelClass = "",
    fullmodal = false,
    children
  }: Props = $props();

  const dispatcher = createEventDispatcher<{
    close: {};
  }>();

  const dialog = createDialog({ label: "Payment Success" });
  onMount(dialog.open);
</script>

<Transition show={isOpen}>
  <div onclose={() => dispatcher("close")}>
    <Transition
      enter="ease-out duration-150 fixed inset-0"
      enterFrom="opacity-0 scale-95"
      enterTo="opacity-100 scale-100"
      leave="ease-in duration-150 fixed inset-0"
      leaveFrom="opacity-100 scale-100"
      leaveTo="opacity-0 scale-95"
    >
      <div class="fixed inset-0 z-10 w-full h-full">
        <div class="relative p-12 w-full h-full">
          <div class="absolute inset-0 z-20 bg-(bg-backdrop opacity-80)"></div>
          <div
            class="relative w-full h-full z-30 m-auto {panelClass} overflow-hidden"
            class:h-full={fullmodal}
            use:dialog.modal
          >
            <div
              class="w-full h-full border-(~ solid border-primary) rounded-lg bg-bg-primary shadow min-h-0 max-h-full"
            >
              {@render children?.()}
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</Transition>
