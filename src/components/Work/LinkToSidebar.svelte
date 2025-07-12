<script lang='ts'>
  import LinkText from '@/components/UI/LinkText.svelte'
  import { query } from '@/store/query'

  interface Props {
    value: string
  }

  const { value }: Props = $props()

  const searchInSidebar = () => {
    query.set(value)
  }
</script>

<LinkText text={value} href={value} intercept on:click={searchInSidebar} />
