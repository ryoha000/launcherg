<script lang='ts'>
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { readImage } from '@tauri-apps/plugin-clipboard-manager'
  import { open } from '@tauri-apps/plugin-dialog'
  import EasyMDE from 'easymde'
  import {
    commandSaveScreenshotByPid,
    commandUploadImage,
  } from '@/lib/command'
  import { showErrorToast } from '@/lib/toast'
  import { memo } from '@/store/memo'
  import { skyWay } from '@/store/skyway'
  import { startProcessMap } from '@/store/startProcessMap'

  const { route }: { route?: { result: { path: { params: { id?: string } } } } } = $props()
  const id = $derived(Number(route?.result?.path?.params?.id || ''))

  let height: number = $state(0)

  const mde = (node: HTMLElement) => {
    const insertImage = (easyMDE: EasyMDE, imagePath: string) => {
      const cursor = easyMDE.codemirror.getCursor()
      const prev = easyMDE.value()
      const lines = prev.split('\n')
      const newLines: string[] = []
      for (let i = 0; i < lines.length; i++) {
        newLines.push(lines[i])
        if (i === cursor.line) {
          newLines.push(`![](${imagePath})`)
          newLines.push('')
        }
      }
      easyMDE.codemirror.setValue(newLines.join('\n'))
      easyMDE.codemirror.setCursor({ line: cursor.line + 2, ch: 0 })
    }
    const easyMDE = new EasyMDE({
      element: node,
      spellChecker: false,
      sideBySideFullscreen: false,
      previewImagesInEditor: true,
      autofocus: true,
      autosave: {
        enabled: true,
        delay: 1000,
        uniqueId: `memo-${id}`,
      },
      toolbar: [
        'bold',
        'italic',
        'heading',
        '|',
        'quote',
        'unordered-list',
        'ordered-list',
        '|',
        'link',
        {
          name: 'image',
          action: async () => {
            const selected = await open({
              multiple: false,
              filters: [
                {
                  name: 'Image',
                  extensions: ['png', 'jpeg', 'jpg', '*'],
                },
              ],
            })
            if (selected === null || Array.isArray(selected)) {
              return
            }
            insertImage(easyMDE, selected)
          },
          className: 'fa fa-picture-o',
          title: 'Insert image',
        },
        {
          name: 'screenshot',
          action: async () => {
            const startProcessId = $startProcessMap[id]
            try {
              const screenshotPath = await commandSaveScreenshotByPid(
                id,
                startProcessId,
              )
              insertImage(easyMDE, screenshotPath)
            }
            catch (e) {
              showErrorToast('スクリーンショットの取得に失敗しました')
              console.error(e)
            }
          },
          className: 'fa fa-desktop',
          title: 'Insert screenshot',
        },
      ],
      imagesPreviewHandler: imagePath => convertFileSrc(imagePath),
    })
    const onPaste = async () => {
      try {
        const image = await readImage()
        const rgba = await image.rgba()
        const size = await image.size()

        const canvas = document.createElement('canvas')
        canvas.width = size.width
        canvas.height = size.height
        const ctx = canvas.getContext('2d')
        if (!ctx) {
          return
        }
        const imageData = new ImageData(
          new Uint8ClampedArray(rgba),
          size.width,
          size.height,
        )
        ctx.putImageData(imageData, 0, 0)
        const base64Image = canvas.toDataURL('image/png').split(',')[1]

        const imagePath = await commandUploadImage(id, base64Image)
        insertImage(easyMDE, imagePath)
      }
      catch {}
    }
    const ele = document.querySelector('.EasyMDEContainer')
    ele?.addEventListener('paste', onPaste)

    const syncTimer = setInterval(() => {
      const current = easyMDE.value()
      if ($memo.find(v => v.workId === id)?.value !== current) {
        memo.update(memos =>
          memos.reduce(
            (acc, cur) => {
              if (cur.workId !== id)
                acc.push(cur)
              return acc
            },
            [
              { workId: id, value: current, lastModified: 'local' },
            ] as typeof $memo,
          ),
        )
        skyWay.syncMemo(id, current)
      }
    }, 1000)

    const unsubscribe = memo.subscribe((memos) => {
      const memo = memos.find(v => v.workId === id)
      if (memo?.lastModified === 'remote' && easyMDE.value() !== memo.value) {
        easyMDE.value(memo.value)
      }
    })
    return {
      destroy: () => {
        ele?.removeEventListener('paste', onPaste)
        unsubscribe()
        clearInterval(syncTimer)
      },
    }
  }
</script>

<div class='h-full min-w-0 w-full' bind:clientHeight={height}>
  <textarea id='mde' use:mde></textarea>
</div>
