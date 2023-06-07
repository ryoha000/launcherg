<script lang="ts">
  import Checkbox from "@/components/UI/Checkbox.svelte";

  export let value: boolean;
  export let label: string;
</script>

<!-- svelte-ignore a11y-label-has-associated-control -->
<label class="flex items-center gap-2 cursor-pointer">
  <Checkbox bind:value on:update />
  <div class="text-(body) transition-all" class:text-text-primary={value}>
    {label}
  </div>
</label>
