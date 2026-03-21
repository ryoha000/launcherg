<script lang='ts'>
  import { onDestroy, onMount } from 'svelte'
  import { preventDefault } from 'svelte/legacy'
  import { fade } from 'svelte/transition'
  import ScanProgressDialog from '@/components/Sidebar/ScanProgressDialog.svelte'
  import { useAutoImport } from '@/components/Sidebar/useAutoImport.svelte'
  import { useImportPaths } from '@/components/Sidebar/useImportPaths.svelte'
  import { useImportProgress } from '@/components/Sidebar/useImportProgress.svelte'
  import Button from '@/components/UI/Button.svelte'
  import Checkbox from '@/components/UI/Checkbox.svelte'
  import InputPath from '@/components/UI/InputPath.svelte'
  import Modal from '@/components/UI/Modal.svelte'

  interface Props {
    isOpen: boolean
  }

  let { isOpen = $bindable() }: Props = $props()

  const {
    paths,
    getPaths,
    updatePath,
    removePath,
    addEmptyPath,
    loadDefaultPaths,
  } = useImportPaths()

  const {
    isLoading,
    useCache,
    setUseCache,
    executeImport,
  } = useAutoImport()

  const {
    progress,
    startListening,
    stopListening,
    beginScan,
    finishScan,
  } = useImportProgress()

  const closeDialog = () => {
    if (isLoading()) {
      return
    }
    isOpen = false
  }

  let inputContainer: HTMLDivElement | null = $state(null)

  const handleAddEmptyPath = () => {
    addEmptyPath(inputContainer)
  }

  const handleConfirm = async () => {
    beginScan()
    try {
      const success = await executeImport(getPaths().map(v => v.path))
      if (success) {
        closeDialog()
      }
    }
    finally {
      finishScan()
    }
  }

  onMount(async () => {
    await loadDefaultPaths()
    await startListening()
  })

  onDestroy(stopListening)
</script>

{#if !isLoading()}
  <Modal
    {isOpen}
    onclose={closeDialog}
    oncancel={closeDialog}
    title='Automatically import game'
    confirmText='Start import'
    fullmodal
    confirmDisabled={!$paths.length || !$paths.some(v => v.path) || isLoading()}
    onconfirm={handleConfirm}
  >
    <div class='space-y-8'>
      <div class='space-y-4'>
        <div class='text-(h4 text-primary) font-medium'>
          自動追加するフォルダ
        </div>
        <form
          class='flex flex-col gap-2'
          onsubmit={preventDefault(handleAddEmptyPath)}
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
                class='ml-auto bg-transparent p-2'
                aria-label='Remove path'
              >
                <div
                  class='i-iconoir-cancel h-5 w-5 color-text-tertiary transition-all hover:color-text-primary'
                ></div>
              </button>
            </div>
          {/each}
          <Button
            appendClass='m-auto'
            leftIcon='i-iconoir-plus'
            text='Add folder path'
            type='submit'
            onclick={handleAddEmptyPath}
          />
        </form>
      </div>
      <div class='space-y-2'>
        <div class='text-(h4 text-primary) font-medium'>オプション</div>
        <label class='flex cursor-pointer gap-2'>
          <Checkbox value={useCache()} on:update={e => setUseCache(e.detail.value)} />
          <div>
            <div class='text-(body text-primary) font-medium'>
              前回から追加されたファイルのみを対象にする
            </div>
            <div class='text-(body2 text-tertiary)'>
              自動追加が初回の場合このオプションは意味を持ちません。このオプションがオフの場合、自動追加は2分程度かかる場合があります。
            </div>
          </div>
        </label>
      </div>
    </div>
  </Modal>
{:else if isLoading()}
  <div transition:fade={{ delay: 150 }}>
    <ScanProgressDialog isOpen={true} progress={progress()} />
  </div>
{/if}
