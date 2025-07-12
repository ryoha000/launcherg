import type { CollectionElement } from '@/lib/types'
import { createLocalStorageWritable } from '@/lib/utils'

export const ATTRIBUTES = {
  NUKIGE: 'nukige',
  NOT_NUKIGE: 'not_nukige',
  LIKE: 'like',
  EXIST_PATH: 'exist_path',
} as const

export const ATTRIBUTE_LABELS: { [key in AttributeKey]: string } = {
  nukige: '抜きゲー',
  not_nukige: '非抜きゲー',
  like: 'お気に入り',
  exist_path: 'パスが存在',
} as const

export type AttributeKey = (typeof ATTRIBUTES)[keyof typeof ATTRIBUTES]
const INITIAL_ATTRIBUTES = Object.values(ATTRIBUTES).map(v => ({
  key: v,
  enabled: false,
}))

export interface Attribute { key: AttributeKey, enabled: boolean }

export function searchAttributes() {
  const [attributes, getAttributes] = createLocalStorageWritable<Attribute[]>(
    'search-attributes',
    INITIAL_ATTRIBUTES,
  )

  const toggleEnable = (key: AttributeKey) => {
    attributes.update((attrs) => {
      const prevIndex = attrs.findIndex(v => v.key === key)
      if (prevIndex < 0) {
        const val = { key, enabled: true }
        // enable: true の最後に追加
        const index = attrs.findLastIndex(v => v.enabled)
        if (index < 0) {
          return [val, ...attrs]
        }
        return [...attrs.slice(0, index), val, ...attrs.slice(index)]
      }

      const val = { key, enabled: !attrs[prevIndex].enabled }
      const removedSelfAttrs = [
        ...attrs.slice(0, prevIndex),
        ...attrs.slice(prevIndex + 1),
      ]
      // enable: true の最後に追加
      const index = removedSelfAttrs.findLastIndex(v => v.enabled)
      if (index < 0) {
        return [val, ...removedSelfAttrs]
      }
      return [
        ...removedSelfAttrs.slice(0, index + 1),
        val,
        ...removedSelfAttrs.slice(index + 1),
      ]
    })
  }

  return {
    attributes: {
      subscribe: attributes.subscribe,
    },
    toggleEnable,
  }
}

export const FILTER_BY_ATTRIBUTE: {
  [key in AttributeKey]: (src: CollectionElement[]) => CollectionElement[];
} = {
  nukige: src => src.filter(v => v.isNukige),
  not_nukige: src => src.filter(v => !v.isNukige),
  exist_path: src => src.filter(v => v.installAt),
  like: src => src.filter(v => v.likeAt),
}
