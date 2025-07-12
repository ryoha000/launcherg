import { commandGetDefaultImportDirs } from '@/lib/command'
import { createLocalStorageWritable } from '@/lib/utils'

export function useImportPaths() {
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

  const addEmptyPath = async (inputContainer: HTMLDivElement | null) => {
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

  const loadDefaultPaths = async () => {
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
  }

  return {
    paths,
    getPaths,
    updatePath,
    removePath,
    addEmptyPath,
    loadDefaultPaths,
  }
}
