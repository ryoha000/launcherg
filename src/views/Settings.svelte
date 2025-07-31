<script lang='ts'>
  import { goto } from '@mateothegreat/svelte5-router'
  import Button from '@/components/UI/Button.svelte'
  import Input from '@/components/UI/Input.svelte'

  let settings = $state({
    theme: 'dark',
    autoLaunch: false,
    showNotifications: true,
    defaultDirectory: '',
    maxRecentGames: '10',
  })

  function saveSettings() {
  // TODO: Tauriコマンドを使って設定を保存
  }

  function resetSettings() {
    settings = {
      theme: 'dark',
      autoLaunch: false,
      showNotifications: true,
      defaultDirectory: '',
      maxRecentGames: '10',
    }
  }

  function navigateToProcTailDebug() {
    goto('/debug/proctail')
  }

  function navigateToExtensionManager() {
    goto('/debug/extensionmanager')
  }
</script>

<div class='p-6 max-w-2xl mx-auto h-full overflow-y-auto'>
  <div class='space-y-6'>
    <!-- テーマ設定 -->
    <div>
      <h2 class='text-lg font-semibold mb-3 text-(text-primary)'>表示設定</h2>
      <div class='space-y-3'>
        <div>
          <label for='theme-select' class='block text-sm font-medium mb-2 text-(text-secondary)'>テーマ</label>
          <select id='theme-select' bind:value={settings.theme} class='w-full p-2 border rounded bg-(bg-secondary) text-(text-primary) border-(border-primary)'>
            <option value='dark'>ダーク</option>
            <option value='light'>ライト</option>
            <option value='auto'>システム設定に従う</option>
          </select>
        </div>
      </div>
    </div>

    <!-- 起動設定 -->
    <div>
      <h2 class='text-lg font-semibold mb-3 text-(text-primary)'>起動設定</h2>
      <div class='space-y-3'>
        <label class='flex items-center'>
          <input type='checkbox' bind:checked={settings.autoLaunch} class='mr-2'>
          <span class='text-(text-secondary)'>システム起動時に自動起動</span>
        </label>
        <label class='flex items-center'>
          <input type='checkbox' bind:checked={settings.showNotifications} class='mr-2'>
          <span class='text-(text-secondary)'>通知を表示</span>
        </label>
      </div>
    </div>

    <!-- ゲーム設定 -->
    <div>
      <h2 class='text-lg font-semibold mb-3 text-(text-primary)'>ゲーム設定</h2>
      <div class='space-y-3'>
        <div>
          <Input bind:value={settings.defaultDirectory} placeholder='ゲームファイルのデフォルトディレクトリ' />
        </div>
        <div>
          <Input bind:value={settings.maxRecentGames} placeholder='10' />
        </div>
      </div>
    </div>

    <!-- 拡張機能 -->
    <div>
      <h2 class='text-lg font-semibold mb-3 text-(text-primary)'>拡張機能</h2>
      <div class='space-y-3'>
        <div class='bg-(bg-secondary) border border-(border-primary) rounded p-4'>
          <h3 class='text-base font-medium mb-2 text-(text-primary)'>ブラウザ拡張機能</h3>
          <p class='text-sm text-(text-secondary) mb-3'>DMM GamesやDLsiteからゲーム情報を自動取得します</p>
          <Button onclick={navigateToExtensionManager} text='拡張機能を管理' />
        </div>
      </div>
    </div>

    <!-- デバッグ設定 -->
    <div>
      <h2 class='text-lg font-semibold mb-3 text-(text-primary)'>デバッグ</h2>
      <div class='space-y-3'>
        <Button variant='normal' onclick={navigateToProcTailDebug} text='ProcTailデバッグ画面' />
      </div>
    </div>

    <!-- ボタン -->
    <div class='flex gap-3 pt-4'>
      <Button onclick={saveSettings} text='設定を保存' />
      <Button variant='normal' onclick={resetSettings} text='リセット' />
    </div>
  </div>
</div>
