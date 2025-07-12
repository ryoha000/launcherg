import type { Readable } from 'svelte/store'
import type { CollectionElement } from '@/lib/types'
import { derived } from 'svelte/store'

export function useVirtualScrollerMasonry(elements: Readable<CollectionElement[]>, setVirtualHeight: (v: number) => void, contentsWidth: Readable<number>, contentsScrollY: Readable<number>, containerHeight: Readable<number>) {
  const minItemWidth = 16 * 16
  const itemGap = 16

  const placeholderHeight = 16 * 8

  type Layout = {
    top: number
    left: number
    width: number
    height: number
    element: CollectionElement
  }[]

  // virtual scroll するために表示されている layout とその上下${buffer}行分を保持する
  const buffer = 5

  // 全ての要素が表示されたときの layout を計算する
  const calculateLayouts = (
    elements: CollectionElement[],
    containerWidth: number,
  ) => {
    if (!containerWidth) {
      return []
    }
    const itemNumPerRow = Math.floor(
      (containerWidth + itemGap) / (minItemWidth + itemGap),
    )
    const itemWidth = Math.floor(
      (containerWidth - itemGap * (itemNumPerRow - 1)) / itemNumPerRow,
    )

    const newLayouts: Layout[] = []

    for (const ele of elements) {
      const itemHeight
        = ele.thumbnailWidth && ele.thumbnailHeight
          ? Math.floor((itemWidth / ele.thumbnailWidth) * ele.thumbnailHeight)
          : placeholderHeight

      if (newLayouts.length < itemNumPerRow) {
        newLayouts.push([
          {
            top: 0,
            left: newLayouts.length * (itemWidth + itemGap),
            width: itemWidth,
            height: itemHeight,
            element: ele,
          },
        ])
      }
      else {
        // それぞれの列について次に要素を追加する top が最小の列を探す
        const lastBottomPerColumn = newLayouts.map((v) => {
          const lastIndex = v.length - 1
          const lastTop = v[lastIndex].top
          const lastHeight = v[lastIndex].height
          return lastTop + lastHeight
        })
        const minBottom = Math.min(...lastBottomPerColumn)
        const minBottomIndex = lastBottomPerColumn.findIndex(
          v => v === minBottom,
        )

        newLayouts[minBottomIndex].push({
          top: minBottom + itemGap,
          left: minBottomIndex * (itemWidth + itemGap),
          width: itemWidth,
          height: itemHeight,
          element: ele,
        })
      }
    }
    return newLayouts
  }

  // layout は 2 次元配列で、各要素は 1 列分の要素を持つ(つまり layouts.length は itemNumPerRow)
  const layouts = derived<[typeof elements, typeof contentsWidth], Layout[]>(
    [elements, contentsWidth],
    ([$elements, $contentsWidth], set) => {
      set(calculateLayouts($elements, $contentsWidth))
    },
  )

  layouts.subscribe((v) => {
    setVirtualHeight(
      Math.max(
        ...v.map((v) => {
          const lastIndex = v.length - 1
          const lastTop = v[lastIndex].top
          const lastHeight = v[lastIndex].height
          return lastTop + lastHeight
        }),
      ),
    )
  })

  const calculateVisibleLayouts = (
    layouts: Layout[],
    scrollTop: number,
    contentsHeight: number,
  ) => {
    const visibleLayouts: Layout = []

    for (let i = 0; i < layouts.length; i++) {
      const layout = layouts[i]
      // この列で最初に一部でも表示されている layout の index
      const firstVisibleIndex = layout.findIndex(
        v => v.top + v.height >= scrollTop,
      )
      // この列で最後に一部でも表示されている layout の index
      let lastVisibleIndex = layout.findIndex(
        v => v.top >= scrollTop + contentsHeight,
      )
      // この列で一部も表示されていない場合は最後の layout が表示されている
      if (lastVisibleIndex === -1) {
        lastVisibleIndex = layout.length - 1
      }
      // 上下${buffer}行分を含めて返す
      const sliceStartIndex = Math.max(firstVisibleIndex - buffer, 0)
      const sliceEndIndex = Math.min(
        lastVisibleIndex + buffer,
        layout.length - 1,
      )

      visibleLayouts.push(...layout.slice(sliceStartIndex, sliceEndIndex + 1))
    }
    return visibleLayouts
  }

  const visibleLayouts = derived<
    [typeof layouts, typeof contentsScrollY, typeof containerHeight],
    Layout
  >(
    [layouts, contentsScrollY, containerHeight] as const,
    ([$layouts, $contentsScrollY, $masonryContainerHeight], set) => {
      set(
        calculateVisibleLayouts(
          $layouts,
          $contentsScrollY,
          $masonryContainerHeight,
        ),
      )
    },
  )

  return {
    visibleLayouts,
  }
}
