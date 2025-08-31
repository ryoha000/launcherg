<script lang='ts'>
  import dlsiteIconUrl from '@/assets/dlsite.ico'
  import dmmIconUrl from '@/assets/dmm.ico'
  import egsIconUrl from '@/assets/erogamescape.ico'
  import { createLinkServiceFilter } from './useLinkServiceFilter'

  export type FilterMode = 'any' | 'linked' | 'unlinked'
  export type StoreFilter = { dmm: FilterMode, dlsite: FilterMode, egs: FilterMode }

  let { filter = $bindable({ dmm: 'any', dlsite: 'any', egs: 'any' }), className, services } = $props<{
    filter: StoreFilter
    className?: string
    services?: Array<{ key: 'dmm' | 'dlsite' | 'egs', label: string, icon: string }>
  }>()

  const { isToggleActive, toggleServiceMode } = createLinkServiceFilter(() => filter, next => (filter = next))

  const serviceList = $derived.by(() => services ?? [
    { key: 'dmm' as const, label: 'DMM', icon: dmmIconUrl },
    { key: 'dlsite' as const, label: 'DLsite', icon: dlsiteIconUrl },
    { key: 'egs' as const, label: 'Erogamescape', icon: egsIconUrl },
  ])

  function modeBtnClass(active: boolean) {
    return active
      ? 'border-(2px accent-accent solid) rounded px-2 py-(0.25) text-(xs text-primary) bg-(bg-secondary) whitespace-nowrap'
      : 'border-(2px transparent solid) rounded px-2 py-(0.25) text-(xs text-secondary) bg-(bg-secondary) whitespace-nowrap'
  }
</script>

<div class={`grid grid-cols-[1rem_auto_auto] gap-2 items-center ${className ?? ''}`}>
  {#each serviceList as s}
    <img src={s.icon} alt={s.label} class='h-4 w-4 object-contain' />
    <span class='text-(sm text-primary)'>{s.label}</span>
    <div class='ml-auto flex items-center gap-1'>
      <button class={modeBtnClass(isToggleActive(filter[s.key], 'linked'))} onclick={() => toggleServiceMode(s.key, 'linked')}>あり</button>
      <button class={modeBtnClass(isToggleActive(filter[s.key], 'unlinked'))} onclick={() => toggleServiceMode(s.key, 'unlinked')}>なし</button>
    </div>
  {/each}
</div>
