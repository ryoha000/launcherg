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

<div class='mx-auto h-full max-w-2xl overflow-y-auto p-6'>
  <div class='space-y-6'>
    <!-- テーマ設定 -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>表示設定</h2>
      <div class='space-y-3'>
        <div>
          <label for='theme-select' class='mb-2 block text-(sm text-secondary) font-medium'>テーマ</label>
          <select id='theme-select' bind:value={settings.theme} class='w-full border border-(border-primary) rounded bg-(bg-secondary) p-2 text-(text-primary)'>
            <option value='dark'>ダーク</option>
            <option value='light'>ライト</option>
            <option value='auto'>システム設定に従う</option>
          </select>
        </div>
      </div>
    </div>

    <!-- 起動設定 -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>起動設定</h2>
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
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>ゲーム設定</h2>
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
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>拡張機能</h2>
      <div class='space-y-3'>
        <div class='border border-(border-primary) rounded bg-(bg-secondary) p-4'>
          <h3 class='mb-2 text-(base text-primary) font-medium'>ブラウザ拡張機能</h3>
          <p class='mb-3 text-(sm text-secondary)'>DMM GamesやDLsiteからゲーム情報を自動取得します</p>
          <Button onclick={navigateToExtensionManager} text='拡張機能を管理' />
        </div>
      </div>
    </div>

    <!-- デバッグ設定 -->
    <div>
      <h2 class='mb-3 text-(lg text-primary) font-semibold'>デバッグ</h2>
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
