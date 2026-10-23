<script lang='ts'>
  import Button from '@/components/UI/Button.svelte'
  import { commandCopyExtensionForDevelopment, commandOpenFolder, commandSetupNativeMessagingHost } from '@/lib/command'

  let busy = $state(false)
  let extensionPath = $state('')
  let error = $state('')

  async function setup() {
    busy = true
    error = ''
    try {
      const path = await commandCopyExtensionForDevelopment()
      await commandSetupNativeMessagingHost()
      extensionPath = path
      await commandOpenFolder(path)
    }
    catch (cause) { error = String(cause) }
    finally { busy = false }
  }
</script>

<section class='space-y-3'>
  <h2 class='text-(lg text-primary) font-semibold'>DMM・DLsite連携（ベータ）</h2>
  <p class='text-(sm text-secondary)'>購入済み作品をブラウザから取り込み、ダウンロードしたゲームを登録します。</p>
  <Button onclick={setup} disabled={busy} text={busy ? '準備中…' : 'ブラウザ連携を準備・更新'} />
  <ol class='list-decimal pl-5 text-(sm text-secondary) space-y-1'>
    <li>上のボタンで拡張機能のフォルダを準備します。</li>
    <li>Chromeの chrome://extensions またはEdgeの edge://extensions を開き、「デベロッパーモード」を有効にします。</li>
    <li>「パッケージ化されていない拡張機能を読み込む」で、開いたフォルダを選びます。更新時は拡張機能の再読み込みを押します。</li>
    <li>DMM・DLsiteにログインし、購入済み作品の一覧を開きます。複数ページある場合は各ページを表示してください。</li>
  </ol>
  <p class='text-(sm text-secondary)'>ダウンロードの元ファイルは保持されます。失敗した場合は拡張機能の「再試行」を使ってください。</p>
  {#if extensionPath}
    <p class='break-all text-(sm text-secondary)'>{extensionPath}</p>
  {/if}
  {#if error}
    <p role='alert' class='text-sm text-red-500'>{error}</p>
  {/if}
</section>
