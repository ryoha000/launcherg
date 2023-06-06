<script>
  import {
    Dialog,
    DialogOverlay,
    Transition,
    TransitionChild,
  } from "@rgossiaux/svelte-headlessui";
  export let isOpen = true;
  export let panelClass = "";
</script>

<Transition show={isOpen}>
  <Dialog on:close={() => (isOpen = false)}>
    <TransitionChild
      enter="ease-out duration-150 fixed inset-0"
      enterFrom="opacity-0"
      enterTo="opacity-100"
      leave="ease-in duration-150 fixed inset-0"
      leaveFrom="opacity-100"
      leaveTo="opacity-0"
    >
      <DialogOverlay class="fixed inset-0 z-10 bg-(bg-backdrop opacity-80) " />
    </TransitionChild>
    <TransitionChild
      enter="ease-out duration-150 fixed inset-0"
      enterFrom="opacity-0 scale-95"
      enterTo="opacity-100 scale-100"
      leave="ease-in duration-150 fixed inset-0"
      leaveFrom="opacity-100 scale-100"
      leaveTo="opacity-0 scale-95"
    >
      <div class="fixed z-20 top-12 left-12 right-12 m-auto {panelClass}">
        <div
          class="w-full h-full border-(~ solid border-primary) rounded-lg bg-bg-primary shadow"
        >
          <slot />
        </div>
      </div>
    </TransitionChild>
  </Dialog>
</Transition>
