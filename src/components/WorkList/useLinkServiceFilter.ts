export type FilterMode = 'any' | 'linked' | 'unlinked'
export type ServiceKey = 'dmm' | 'dlsite' | 'egs'
export interface StoreFilter { dmm: FilterMode, dlsite: FilterMode, egs: FilterMode }

export function createLinkServiceFilter(
  getFilter: () => StoreFilter,
  setFilter: (next: StoreFilter) => void,
) {
  type ToggleTarget = 'linked' | 'unlinked'

  function isToggleActive(mode: FilterMode, target: ToggleTarget) {
    return mode === 'any' || mode === target
  }

  function toggleServiceMode(service: ServiceKey, target: ToggleTarget) {
    const current = getFilter()
    const mode = current[service]
    const linkedActive = mode === 'any' || mode === 'linked'
    const unlinkedActive = mode === 'any' || mode === 'unlinked'
    let nextLinked = linkedActive
    let nextUnlinked = unlinkedActive
    if (target === 'linked')
      nextLinked = !linkedActive
    else nextUnlinked = !unlinkedActive
    let nextMode: FilterMode
    if (nextLinked && nextUnlinked)
      nextMode = 'any'
    else if (nextLinked)
      nextMode = 'linked'
    else if (nextUnlinked)
      nextMode = 'unlinked'
    else nextMode = 'any'
    setFilter({ ...current, [service]: nextMode })
  }

  return { isToggleActive, toggleServiceMode }
}
