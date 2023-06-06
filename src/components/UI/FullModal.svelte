<script>
  import {
    Dialog,
    DialogOverlay,
    Transition,
    TransitionChild,
  } from "@rgossiaux/svelte-headlessui";
  export let isOpen = true;
</script>

<Transition show={isOpen}>
  <Dialog on:close={() => (isOpen = false)}>
    <TransitionChild
      enter="ease-out duration-150"
      enterFrom="opacity-0"
      enterTo="opacity-100"
      leave="ease-in duration-150"
      leaveFrom="opacity-100"
      leaveTo="opacity-0"
    >
      <DialogOverlay class="fixed inset-0 z-10 bg-(bg-backdrop opacity-80) " />
    </TransitionChild>
    <TransitionChild
      enter="ease-out duration-150"
      enterFrom="opacity-0 scale-95"
      enterTo="opacity-100 scale-100"
      leave="ease-in dduration-150"
      leaveFrom="opacity-100 scale-100"
      leaveTo="opacity-0 scale-95"
    >
      <div class="fixed inset-12 z-20">
        <div
          class="w-full h-full border-(~ solid border-primary) rounded-lg bg-bg-primary shadow"
        >
          <slot />
        </div>
      </div>
    </TransitionChild>
  </Dialog>
</Transition>
