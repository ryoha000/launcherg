<script lang="ts">
  import Button from "@/components/UI/Button.svelte";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
</script>

<div class="flex items-center min-w-0">
  <Button
    appendClass="w-24 h-8 flex items-center rounded-r-0 group hover:text-bg-success-hover transition-all"
    leftIcon="i-material-symbols-power-rounded group-hover:color-bg-success-hover transition-all"
    text="Play"
    on:click
  />
  <ButtonBase
    appendClass="h-8 w-8 flex items-center justify-center rounded-l-0"
    tooltip={{
      content: "起動についての設定",
      placement: "bottom",
      theme: "default",
    }}
    on:click
  >
    <div class="color-ui-tertiary w-5 h-5 i-material-symbols-tune" />
  </ButtonBase>
</div>
