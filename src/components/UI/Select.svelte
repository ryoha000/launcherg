<script lang='ts'>
  import type { Option } from '@/lib/trieFilter'
  import { createEventDispatcher } from 'svelte'
  import APopover from '@/components/UI/APopover.svelte'
  import SelectOptions from '@/components/UI/SelectOptions.svelte'

  interface Props {
    options: Option<string | number>[]
    value: Option<string | number>['value']
    iconClass?: string
    title?: string | undefined
    enableFilter?: boolean
    filterPlaceholder?: string
    bottomCreateButtonText?: string
    showSelectedCheck?: boolean
    children?: import('svelte').Snippet
  }

  let {
    options,
    value = $bindable(),
    iconClass = '',
    title = undefined,
    enableFilter = false,
    filterPlaceholder = '',
    bottomCreateButtonText = '',
    showSelectedCheck = false,
    children,
  }: Props = $props()

  const selectedLabel = $derived(options.find(v => v.value === value)?.label ?? '')

  const dispather = createEventDispatcher<{ create: {} }>()

  const children_render = $derived(children)
</script>

<APopover>
  {#snippet button()}
    <div>
      {#if children_render}{@render children_render()}{:else}
    <button
      class='h-8 w-full flex items-center gap-2 border border-(border-button opacity-10 solid) rounded bg-bg-button px-3 transition-all hover:(border-border-button-hover bg-bg-button-hover) overflow-hidden'
    >
      {#if iconClass}
        <div class={`${iconClass} w-4 h-4`}></div>
      {/if}
      <div class='text-(body text-primary) font-bold max-h-full'>
        {selectedLabel}
      </div>
      <div
        class='i-material-symbols-arrow-drop-down ml-auto h-4 w-4 color-text-primary transition-all flex-shrink-0'
        class:rotate-180={open}
      ></div>
    </button>
  {/if}
    </div>
  {/snippet}
  {#snippet children({ open, close })}
    <SelectOptions
      {title}
      {enableFilter}
      {filterPlaceholder}
      {options}
      {bottomCreateButtonText}
      {showSelectedCheck}
      bind:value
      on:select
      on:create={() => {
        close(null)
        dispather('create')
      }}
      on:close={() => close(null)}
    />
  {/snippet}
</APopover>
