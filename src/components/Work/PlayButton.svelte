<script lang="ts">
  import APopover from "@/components/UI/APopover.svelte";
  import Button from "@/components/UI/Button.svelte";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import PlayPopover from "@/components/Work/PlayPopover.svelte";
  import { createEventDispatcher } from "svelte";

  const dispather = createEventDispatcher<{
    play: { isAdmin: boolean | undefined };
  }>();
</script>

<div class="flex items-center min-w-0">
  <Button
    appendClass="rounded-r-0"
    leftIcon="i-material-symbols-power-rounded"
    text="Play"
    variant="success"
    on:click={() => dispather("play", { isAdmin: undefined })}
  />
  <APopover  >
    {#snippet button()}
        <ButtonBase
        appendClass="h-8 w-8 flex items-center justify-center rounded-l-0"
        tooltip={{
          content: "このゲームの設定",
          placement: "bottom",
          theme: "default",
          delay: 1000,
        }}
        variant="success"
        
      >
        <div
          class="color-text-white w-5 h-5 i-material-symbols-arrow-drop-down"
          class:rotate-180={open}
  ></div>
      </ButtonBase>
      {/snippet}
    {#snippet children({ open, close })}
        <PlayPopover
        on:close={() => close(null)}
        on:play={() => {
          dispather("play", { isAdmin: false });
        }}
        on:playAdmin={() => {
          dispather("play", { isAdmin: true });
        }}
      />
          {/snippet}
    </APopover>
</div>
