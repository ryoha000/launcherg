<script lang="ts">
  import {
    Dialog,
    DialogOverlay,
    Transition,
    TransitionChild,
  } from "@rgossiaux/svelte-headlessui";
  import { createEventDispatcher } from "svelte";
  export let isOpen = true;
  export let panelClass = "";
  export let fullmodal = false;

  const dispatcher = createEventDispatcher<{
    close: {};
  }>();
</script>

<Transition show={isOpen}>
  <Dialog on:close={() => dispatcher("close")}>
    <TransitionChild
      enter="ease-out duration-150 fixed inset-0"
      enterFrom="opacity-0 scale-95"
      enterTo="opacity-100 scale-100"
      leave="ease-in duration-150 fixed inset-0"
      leaveFrom="opacity-100 scale-100"
      leaveTo="opacity-0 scale-95"
    >
      <div class="fixed inset-0 z-10 w-full h-full">
        <div class="relative p-12 w-full h-full">
          <DialogOverlay
            class="absolute inset-0 z-20 bg-(bg-backdrop opacity-80)"
          />
          <div
            class="relative w-full h-full z-30 m-auto {panelClass} overflow-hidden"
            class:h-full={fullmodal}
          >
            <div
              class="w-full h-full border-(~ solid border-primary) rounded-lg bg-bg-primary shadow min-h-0 max-h-full"
            >
              <slot />
            </div>
          </div>
        </div>
      </div>
    </TransitionChild>
  </Dialog>
</Transition>
