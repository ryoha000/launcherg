<script lang='ts'>
  import type { AllGameCacheOne } from '@/lib/types'

  import { run } from 'svelte/legacy'
  import Input from '@/components/UI/Input.svelte'
  import InputPath from '@/components/UI/InputPath.svelte'
  import Modal from '@/components/UI/Modal.svelte'
  import { commandGetGameCandidates } from '@/lib/command'
  import { useImportManually } from '@/lib/importManually'

  interface Props {
    isOpen: boolean
    withInputPath?: boolean
    cancelText?: string
    idInput?: string
    path?: string
    onconfirm?: (exePath: string | null, lnkPath: string | null, gameCache: AllGameCacheOne) => void
    oncancel?: () => void
  }

  let {
    isOpen = $bindable(),
    withInputPath = true,
    cancelText = 'Cancel',
    idInput = $bindable(''),
    path = $bindable(''),
    onconfirm,
    oncancel,
  }: Props = $props()

  let candidates: [number, string][] = $state([])
  run(() => {
    (async () => {
      if (!path) {
        candidates = []
        return
      }
      candidates = await commandGetGameCandidates(path)
    })()
  })
  const clickCandidate = (id: number) => {
    idInput = `${id}`
  }

  const { getNewCollectionElementByInputs } = useImportManually()
  const onConfirm = async () => {
    const val = await getNewCollectionElementByInputs(idInput, path)
    if (val) {
      onconfirm?.(val.exePath, val.lnkPath, val.gameCache)
    }
  }
</script>

<Modal
  {isOpen}
  onclose={() => (isOpen = false)}
  {oncancel}
  title='Manually import game'
  confirmText='Import'
  {cancelText}
  confirmDisabled={!idInput || (!path && withInputPath)}
  onconfirm={onConfirm}
>
  <div class='space-y-4'>
    {#if withInputPath}
      <InputPath
        {path}
        label='EXEファイル or ショットカットファイル のパス'
        placeholder='C:\game\Monkeys!!\Monkeys!!.exe'
        on:update={e => (path = e.detail.value)}
      />
    {/if}
    <div class='space-y-2'>
      <Input
        bind:value={idInput}
        label='ErogameScape ID or URL'
        placeholder='17909 or https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/game.php?game=17909'
        on:update={e => (idInput = e.detail.value)}
      />
      {#if candidates.length !== 0}
        <div class='space-y-1 pl-2'>
          <h4 class='text-(text-primary body) font-medium mb-1'>候補</h4>
          <div class='w-full'>
            {#each candidates as [id, gamename] (id)}
              <button
                class={`rounded hover:bg-bg-button-hover transition-all px-4 block max-w-full ${
                  idInput === `${id}` ? 'bg-bg-button' : 'bg-inherit'
                }`}
                onclick={() => clickCandidate(id)}
              >
                <div
                  class='text-(text-secondary left body2) overflow-ellipsis whitespace-nowrap overflow-hidden w-full'
                >
                  {gamename}
                </div>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
</Modal>
