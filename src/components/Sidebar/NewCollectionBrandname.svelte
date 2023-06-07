<script lang="ts">
  import Select from "@/components/UI/Select.svelte";
  import { createEventDispatcher } from "svelte";

  export let brandnames: string[];
  export let value: string;
  const brandnameOptions = brandnames.map((v) => ({ label: v, value: v }));

  const dispatcher = createEventDispatcher<{ change: { value: string } }>();

  $: {
    console.log("diaptch change", value);
    dispatcher("change", { value });
  }
</script>

<Select
  bind:value
  enableFilter
  options={brandnameOptions}
  title="Select filter brandname"
/>
