<script lang='ts'>
  import { listen } from '@tauri-apps/api/event'

  import { onMount } from 'svelte'
  import { preventDefault } from 'svelte/legacy'
  import { fade } from 'svelte/transition'
  import Button from '@/components/UI/Button.svelte'
  import Checkbox from '@/components/UI/Checkbox.svelte'
  import InputPath from '@/components/UI/InputPath.svelte'
  import Modal from '@/components/UI/Modal.svelte'
  import ModalBase from '@/components/UI/ModalBase.svelte'
  import {
    commandCreateElementsInPc,
    commandGetDefaultImportDirs,
  } from '@/lib/command'
  import { registerCollectionElementDetails } from '@/lib/registerCollectionElementDetails'
  import { showInfoToast } from '@/lib/toast'
  import { createLocalStorageWritable } from '@/lib/utils'
  import { sidebarCollectionElements } from '@/store/sidebarCollectionElements'

  let isLoading = $state(false)

  interface Props {
    isOpen: boolean
  }

  let { isOpen = $bindable() }: Props = $props()

  let inputContainer: HTMLDivElement | null = $state(null)

  let useCache = $state(true)
  const [paths, getPaths] = createLocalStorageWritable<
    { id: number, path: string }[]
  >('auto-import-dir-paths', [
    { id: Math.floor(Math.random() * 100000), path: '' },
  ])
  const updatePath = (index: number, value: string) => {
    paths.update((v) => {
      v[index].path = value
      return v
    })
  }
  const removePath = (index: number) => {
    paths.update((v) => {
      v = [...v.slice(0, index), ...v.slice(index + 1)]
      return v
    })
  }
  const addEmptyPath = async () => {
    if (
      getPaths().length > 0
      && getPaths()[getPaths().length - 1].path === ''
    ) {
      return
    }
    paths.update((v) => {
      v.push({ id: new Date().getTime(), path: '' })
      return v
    })
    await new Promise(resolve => setTimeout(resolve, 0))
    if (inputContainer) {
      const inputs = inputContainer.getElementsByTagName('input')
      if (inputs.length > 0) {
        inputs[inputs.length - 1].focus()
      }
    }
  }
  const confirm = async () => {
    isLoading = true
    const res = await commandCreateElementsInPc(
      getPaths().map(v => v.path),
      useCache,
    )
    await registerCollectionElementDetails()
    await sidebarCollectionElements.refetch()

    isLoading = false

    const text = res.length
      ? `「${res[0]}」${
        res.length === 1 ? 'が' : `、他${res.length}件`
      }追加されました`
      : '新しく追加されたゲームはありません'

    showInfoToast(text)
    isOpen = false
  }

  let processFileNums = $state(0)
  let processedFileNums = $state(0)

  onMount(async () => {
    const defaultPaths = await commandGetDefaultImportDirs()
    paths.update((v) => {
      const appendPaths = []
      for (const defaultPath of defaultPaths) {
        if (!v.some(v => v.path === defaultPath)) {
          appendPaths.push({
            id: Math.floor(Math.random() * 100000),
            path: defaultPath,
          })
        }
      }
      return [...appendPaths, ...v]
    })
    // const unlistenProgress = await listen<{ message: string }>(
    //   "progress",
    //   (event) => {
    //     showInfoToast(event.payload.message, 10000);
    //   }
    // );
    const unlistenProgressLive = await listen<{ max: number | null }>(
      'progresslive',
      (event) => {
        if (event.payload.max) {
          processFileNums = event.payload.max
        }
        else {
          processedFileNums = processedFileNums + 1
        }
      },
    )
    return () => {
      // unlistenProgress();
      unlistenProgressLive()
    }
  })
</script>

{#if !isLoading}
  <Modal
    {isOpen}
    on:close={() => {
      if (!isLoading) {
        isOpen = false
      }
    }}
    on:cancel={() => {
      if (!isLoading) {
        isOpen = false
      }
    }}
    title='Automatically import game'
    confirmText='Start import'
    fullmodal
    confirmDisabled={!$paths.length || !$paths.some(v => v.path) || isLoading}
    on:confirm={confirm}
  >
    <div class='space-y-8'>
      <div class='space-y-4'>
        <div class='text-(text-primary h4) font-medium'>
          自動追加するフォルダ
        </div>
        <form
          class='flex flex-col gap-2'
          onsubmit={preventDefault(addEmptyPath)}
        >
          {#each $paths as path, i (path.id)}
            <div class='flex items-end gap-8' bind:this={inputContainer}>
              <div class='flex-1'>
                <InputPath
                  label=""
                  placeholder='C:\Program Files (x86)'
                  path={path.path}
                  directory
                  withFilter={false}
                  on:update={e => updatePath(i, e.detail.value)}
                />
              </div>
              <button
                onclick={() => removePath(i)}
                type='button'
                tabindex={-1}
                class='ml-auto p-2 bg-transparent'
              >
                <div
                  class='w-5 h-5 i-iconoir-cancel color-text-tertiary hover:color-text-primary transition-all'
                ></div>
              </button>
            </div>
          {/each}
          <Button
            appendClass='m-auto'
            leftIcon='i-iconoir-plus'
            text='Add folder path'
            type='submit'
            on:click={addEmptyPath}
          />
        </form>
      </div>
      <div class='space-y-2'>
        <div class='text-(text-primary h4) font-medium'>オプション</div>
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class='flex gap-2 cursor-pointer'>
          <Checkbox bind:value={useCache} />
          <div>
            <div class='text-(text-primary body) font-medium'>
              前回から追加されたファイルのみを対象にする
            </div>
            <div class='text-(text-tertiary body2)'>
              自動追加が初回の場合このオプションは意味を持ちません。このオプションがオフの場合、自動追加は2分程度かかる場合があります。
            </div>
          </div>
        </label>
      </div>
    </div>
  </Modal>
{:else if isLoading}
  <div transition:fade={{ delay: 150 }}>
    <ModalBase isOpen={true} panelClass='max-w-82'>
      <div class='flex-(~ col) items-center justify-center gap-5 w-full p-12'>
        <div
          class='w-20 h-20 border-(12px solid #D9D9D9 t-#2D2D2D t-rounded) rounded-full animate-spin'
        ></div>
        <div class='text-(text-primary h3) font-bold'>処理中</div>
        {#if processFileNums}
          <div class='text-(text-primary body) font-medium'>
            処理したファイル: {processedFileNums}/{processFileNums}
          </div>
        {/if}
      </div>
    </ModalBase>
  </div>
{/if}
