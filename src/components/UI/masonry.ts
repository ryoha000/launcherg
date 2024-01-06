import type { CollectionElement } from "@/lib/types";
import { createWritable } from "@/lib/utils";
import { writable, type Readable, type Writable, derived } from "svelte/store";

export const useMasonry = (elements: CollectionElement[]) => {
  const minItemWidth = 16 * 16;
  const itemGap = 16;

  const placeholderHeight = 16 * 8;

  type Layout = {
    top: number;
    left: number;
    width: number;
    height: number;
    element: CollectionElement;
  }[];
  const [layouts, layoutsValue] = createWritable<Layout[]>([]); // layout は 2 次元配列で、各要素は 1 列分の要素を持つ(つまり layouts.length は itemNumPerRow)

  const virtualHeight = derived<[typeof layouts], number>(
    [layouts],
    ([$layouts], set) => {
      set(
        Math.max(
          ...$layouts.map((v) => {
            const lastIndex = v.length - 1;
            const lastTop = v[lastIndex].top;
            const lastHeight = v[lastIndex].height;
            return lastTop + lastHeight;
          })
        )
      );
    }
  );

  // virtual scroll するために表示されている layout とその上下${buffer}行分を保持する
  const buffer = 5;
  const [scrollY, scrollYValue] = createWritable(0);
  const [masonryHeaderHeight, masonryHeaderHeightValue] = createWritable(0);
  const [masonryContentsWidth, masonryContentsWidthValue] = createWritable(0);
  const [masonryContentsHeight, masonryContentsHeightValue] = createWritable(0);
  const masonryContainerHeight = writable(0);

  const masonryHeader = (node: HTMLElement) => {
    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) {
        return;
      }
      masonryHeaderHeight.set(entry.contentRect.height);
    });
    resizeObserver.observe(node);

    return {
      destroy() {
        resizeObserver.disconnect();
      },
    };
  };
  const masonryContents = (node: HTMLElement) => {
    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) {
        return;
      }
      masonryContentsWidth.set(entry.contentRect.width);
      masonryContentsHeight.set(entry.contentRect.height);
    });
    resizeObserver.observe(node);

    return {
      destroy() {
        resizeObserver.disconnect();
      },
    };
  };

  const masonry = (node: HTMLElement) => {
    const calculateLayouts = (
      elements: CollectionElement[],
      containerWidth: number
    ) => {
      if (!containerWidth) {
        return [];
      }
      const itemNumPerRow = Math.floor(
        (containerWidth + itemGap) / (minItemWidth + itemGap)
      );
      const itemWidth = Math.floor(
        (containerWidth - itemGap * (itemNumPerRow - 1)) / itemNumPerRow
      );

      const newLayouts: Layout[] = [];

      for (const ele of elements) {
        const itemHeight =
          ele.thumbnailWidth && ele.thumbnailHeight
            ? Math.floor((itemWidth / ele.thumbnailWidth) * ele.thumbnailHeight)
            : placeholderHeight;

        if (newLayouts.length < itemNumPerRow) {
          newLayouts.push([
            {
              top: 0,
              left: newLayouts.length * (itemWidth + itemGap),
              width: itemWidth,
              height: itemHeight,
              element: ele,
            },
          ]);
        } else {
          // それぞれの列について次に要素を追加する top が最小の列を探す
          const lastBottomPerColumn = newLayouts.map((v) => {
            const lastIndex = v.length - 1;
            const lastTop = v[lastIndex].top;
            const lastHeight = v[lastIndex].height;
            return lastTop + lastHeight;
          });
          const minBottom = Math.min(...lastBottomPerColumn);
          const minBottomIndex = lastBottomPerColumn.findIndex(
            (v) => v === minBottom
          );

          newLayouts[minBottomIndex].push({
            top: minBottom + itemGap,
            left: minBottomIndex * (itemWidth + itemGap),
            width: itemWidth,
            height: itemHeight,
            element: ele,
          });
        }
      }
      console.log({ newLayouts });
      return newLayouts;
    };

    // TODO: 確認
    const initailLayouts = calculateLayouts(
      elements,
      masonryContentsWidthValue()
    );
    layouts.set(initailLayouts);

    const onScroll = (e: Event) => {
      const target = e.target;
      if (!(target instanceof HTMLElement)) {
        return;
      }
      // debugger;
      console.log({ scrollTop: target.scrollTop });
      scrollY.set(target.scrollTop);
    };
    node.addEventListener("scroll", onScroll);
    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry) {
        return;
      }
      masonryContainerHeight.set(entry.contentRect.height);
      layouts.set(calculateLayouts(elements, masonryContentsWidthValue()));
    });
    resizeObserver.observe(node);

    masonryContentsWidth.subscribe((width) => {
      layouts.set(calculateLayouts(elements, width));
    });

    return {
      destroy() {
        node.removeEventListener("scroll", onScroll);
        resizeObserver.disconnect();
      },
    };
  };

  const calculateVisibleLayouts = (
    layouts: Layout[],
    scrollTop: number,
    contentsHeight: number
  ) => {
    const visibleLayouts: Layout = [];

    for (let i = 0; i < layouts.length; i++) {
      const layout = layouts[i];
      // この列で最初に一部でも表示されている layout の index
      const firstVisibleIndex = layout.findIndex(
        (v) => v.top + v.height >= scrollTop
      );
      // この列で最後に一部でも表示されている layout の index
      const lastVisibleIndex = layout.findIndex(
        (v) => v.top >= scrollTop + contentsHeight
      );
      // // この列で全く表示されていない場合はスキップ
      // if (firstVisibleIndex === -1 && lastVisibleIndex === -1) {
      //   continue;
      // }
      // 上下1行分を含めて返す
      const sliceStartIndex = Math.max(firstVisibleIndex - buffer, 0);
      const sliceEndIndex = Math.min(
        lastVisibleIndex + buffer,
        layout.length - 1
      );

      visibleLayouts.push(...layout.slice(sliceStartIndex, sliceEndIndex));
    }
    console.log({ visibleLayouts, scrollTop, contentsHeight });
    return visibleLayouts;
  };

  const visibleLayouts = derived<
    [
      typeof layouts,
      typeof scrollY,
      typeof masonryContainerHeight,
      typeof masonryHeaderHeight
    ],
    Layout
  >(
    [layouts, scrollY, masonryContainerHeight, masonryHeaderHeight] as const,
    (
      [$layouts, $scrollY, $masonryContainerHeight, $masonryHeaderHeight],
      set
    ) => {
      set(
        calculateVisibleLayouts(
          $layouts,
          Math.max(0, $scrollY - $masonryHeaderHeight),
          $masonryContainerHeight
        )
      );
    }
  );

  return {
    masonry,
    masonryHeader,
    masonryContents,
    visibleLayouts,
    virtualHeight,
  };
};
