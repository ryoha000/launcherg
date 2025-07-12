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

<div class='border-(~ solid border-primary) rounded-xl'>
  <div class='bg-bg-secondary p-4 rounded-t-xl'>
    <div class='text-(body text-primary)'>{title}</div>
  </div>
  <div class='grid-(~ cols-[min-content_1fr])'>
    {#each rows as row (row.label)}
      <div
        class='p-4 border-(t-1px solid border-primary) whitespace-nowrap text-(body2 text-primary)'
      >
        {row.label}
      </div>
      {#if row.component}
        <div class='p-4 border-(t-1px solid border-primary)'>
          <row.component value={row.value} />
        </div>
      {:else}
        <div
          class='p-4 border-(t-1px solid border-primary) text-(body2 text-primary)'
        >
          {row.value}
        </div>
      {/if}
    {/each}
  </div>
</div>
