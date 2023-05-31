<script lang="ts">
  import ButtonBase from "@/components/UI/ButtonBase.svelte";

  export let icon: string;
</script>

<ButtonBase appendClass="h-8 w-8 p-2" on:click>
  <div class={`color-ui-tertiary w-4 h-4 ${icon}`} />
</ButtonBase>
