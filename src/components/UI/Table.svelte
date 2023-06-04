<script lang="ts">
  export let title: string;
  export let rows: {
    label: string;
    value: string | { text: string; link: string };
  }[];
</script>

<div class="border-(~ solid border-primary) rounded-xl">
  <div class="bg-bg-secondary p-4 rounded-t-xl">
    <div class="text-(body text-primary)">{title}</div>
  </div>
  <div class="grid-(~ cols-[min-content_1fr]) items-center">
    {#each rows as row (row.label)}
      <div
        class="p-4 border-(t-1px solid border-primary) whitespace-nowrap text-(body2 text-primary)"
      >
        {row.label}
      </div>
      {#if typeof row.value.link === "string"}
        <div
          class="p-4 border-(t-1px solid border-primary) text-(body2 text-link)"
        >
          {row.value}
        </div>
      {:else}
        <div
          class="p-4 border-(t-1px solid border-primary) text-(body2 text-primary)"
        >
          {row.value}
        </div>
      {/if}
    {/each}
  </div>
</div>
