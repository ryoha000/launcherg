<script lang='ts'>
  interface Props {
    title: string
    rows: {
      label: string
      value: string
      component?: any
    }[]
  }

  const { title, rows }: Props = $props()
</script>

<div class='border border-(border-primary solid) rounded-xl'>
  <div class='rounded-t-xl bg-bg-secondary p-4'>
    <div class='text-(body text-primary)'>{title}</div>
  </div>
  <div class='grid grid-(cols-[min-content_1fr])'>
    {#each rows as row (row.label)}
      <div
        class='whitespace-nowrap border-(t-1px border-primary solid) p-4 text-(body2 text-primary)'
      >
        {row.label}
      </div>
      {#if row.component}
        <div class='border-(t-1px border-primary solid) p-4'>
          <row.component value={row.value} />
        </div>
      {:else}
        <div
          class='border-(t-1px border-primary solid) p-4 text-(body2 text-primary)'
        >
          {row.value}
        </div>
      {/if}
    {/each}
  </div>
</div>
