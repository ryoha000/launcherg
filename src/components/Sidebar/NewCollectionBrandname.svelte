<script lang="ts">
  import Select from "@/components/UI/Select.svelte";
  import { createEventDispatcher } from "svelte";

  export let brandnameAndRubies: [string, string][];
  export let value: string;
  const brandnameOptions = brandnameAndRubies.map(([name, ruby]) => ({
    label: name,
    value: name,
    otherLabels: [ruby],
  }));

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
