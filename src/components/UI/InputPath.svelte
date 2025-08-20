<script lang='ts'>
  import { open } from '@tauri-apps/plugin-dialog'
  import { createEventDispatcher } from 'svelte'
  import Button from '@/components/UI/Button.svelte'
  import Input from '@/components/UI/Input.svelte'

  interface Props {
    path: string
    label: string
    placeholder?: string
    withFilter?: boolean
    directory?: boolean
  }

  let {
    path = $bindable(),
    label,
    placeholder = '',
    withFilter = true,
    directory = false,
  }: Props = $props()

  const dispatcher = createEventDispatcher<{ update: { value: string } }>()

  const openDialog = async () => {
    const selected = await open({
      multiple: false,
      filters: withFilter
        ? [
          {
            name: 'exe',
            extensions: ['exe', 'EXE', 'lnk'],
          },
        ]
        : [],
      directory,
    })
    if (selected === null || Array.isArray(selected)) {
      return
    }
    path = selected
    dispatcher('update', { value: selected })
  }
</script>

<div class='flex items-end gap-2'>
  <div class='flex-1'>
    <Input value={path} {label} {placeholder} on:update />
  </div>
  <Button
    leftIcon='i-material-symbols-folder-outline-rounded'
    onclick={openDialog}
  />
</div>
