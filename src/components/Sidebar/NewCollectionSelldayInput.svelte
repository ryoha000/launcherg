<script lang="ts">
  import Input from "@/components/UI/Input.svelte";

  export let label: string;
  export let year: string;
  export let month: string;
</script>

<div>
  <div class="text-(body text-primary)">{label}</div>
  <div class="flex items-center gap-2 max-w-80">
    <Input bind:value={year} placeholder={"2020"} />
    <div class="text-(body2 text-primary)">年</div>
    <Input bind:value={month} placeholder={"10"} />
    <div class="text-(body2 text-primary)">月</div>
  </div>
</div>
