<script lang='ts'>
  import ButtonIcon from '@/components/UI/ButtonIcon.svelte'
  import { commandOpenFolder } from '@/lib/command'
  import { showInfoToast } from '@/lib/toast'

  interface PathItem {
    label: string
    path?: string | null
  }

  interface Props {
    items: PathItem[]
  }

  const { items }: Props = $props()

  const copyPath = async (path?: string | null) => {
    if (!path) {
      return
    }
    await navigator.clipboard.writeText(path)
    showInfoToast('クリップボードにコピーしました')
  }

  const openPath = async (path?: string | null) => {
    if (!path) {
      return
    }
    await commandOpenFolder(path)
  }

  const rowClass = 'flex min-w-0 items-center gap-2 rounded px-2 py-1'
  // truncate を使うことで正しく text-overflow: ellipsis にする
  // ツールチップ付きボタン(2つ)+ギャップ等で約 4.5rem 分をホバー時に確保して、...がボタンの下に隠れないようにする
  const valueClass = 'truncate min-w-0 flex-1 text-(body2 text-secondary) transition-all group-hover:pr-[4.5rem] group-focus-within:pr-[4.5rem]'
</script>

<div class='grid grid-cols-[max-content_minmax(0,1fr)] gap-x-4'>
  {#each items as item}
    <div class='self-center whitespace-nowrap text-(body2 text-primary)'>
      {item.label}
    </div>
    <div class='min-w-0'>
      {#if item.path}
        <div class="{rowClass} group relative transition-all">
          <div class={valueClass}>
            {item.path}
          </div>
          <div class='absolute right-2 top-1/2 flex -translate-y-1/2 items-center gap-1 pl-2 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100'>
            <ButtonIcon
              icon='i-material-symbols-content-copy-outline-rounded'
              onclick={() => copyPath(item.path)}
              tooltip={{ content: 'コピー' }}
            />
            <ButtonIcon
              icon='i-material-symbols-folder-open-rounded'
              onclick={() => openPath(item.path)}
              tooltip={{ content: '開く' }}
            />
          </div>
        </div>
      {:else}
        <div class={rowClass}>
          <div class={valueClass}>
            未設定
          </div>
        </div>
      {/if}
    </div>
  {/each}
</div>
