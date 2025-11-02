import type { Descriptor } from './registry'
import { keyedTab, pathParamExtractor, queryParamExtractor, singletonTab } from '@/store/tabs/schema'
import DenyList from '@/views/Debug/DenyList.svelte'
import ExtensionLog from '@/views/Debug/ExtensionLog.svelte'
import ExtensionManager from '@/views/Debug/ExtensionManager.svelte'
import ProcTail from '@/views/Debug/ProcTail.svelte'
import Home from '@/views/Home.svelte'
import ImageQueue from '@/views/ImageQueue.svelte'
import Memo from '@/views/Memo.svelte'
import Settings from '@/views/Settings.svelte'
import StoreMapped from '@/views/StoreMapped.svelte'
import Work from '@/views/Work.svelte'

export const ROUTE_REGISTRY = [
  {
    kind: 'home',
    pathTemplate: '/',
    component: Home,
    tab: { mode: 'none' },
  },
  {
    kind: 'works',
    pathTemplate: '/works/:id(.*?)',
    component: Work,
    icon: 'i-material-symbols-computer-outline-rounded color-accent-accent',
    tab: keyedTab(pathParamExtractor('id'), queryParamExtractor('gamename')),
  },
  {
    kind: 'memos',
    pathTemplate: '/memos/:id(.*?)',
    component: Memo,
    icon: 'i-material-symbols-drive-file-rename-outline color-accent-edit',
    tab: keyedTab(
      pathParamExtractor('id'),
      queryParamExtractor('gamename', v => `メモ - ${v}`),
    ),
  },
  {
    kind: 'settings',
    pathTemplate: '/settings',
    component: Settings,
    icon: 'i-material-symbols-settings-outline-rounded color-text-disabled',
    tab: singletonTab('設定'),
  },
  {
    kind: 'image-queue',
    pathTemplate: '/image-queue',
    component: ImageQueue,
    icon: 'i-material-symbols-image-outline color-text-disabled',
    tab: singletonTab('画像保存キュー'),
  },
  {
    kind: 'store-mapped',
    pathTemplate: '/store-mapped',
    component: StoreMapped,
    icon: 'i-material-symbols-checklist color-text-disabled',
    tab: singletonTab('ダウンロード購入作品の管理'),
  },

  // Debug
  {
    kind: 'debug-proctail',
    pathTemplate: '/debug/proctail',
    component: ProcTail,
    icon: 'i-material-symbols-bug-report-outline-rounded color-text-tertiary',
    tab: singletonTab('proctail デバッグ'),
  },
  {
    kind: 'debug-extensionmanager',
    pathTemplate: '/debug/extensionmanager',
    component: ExtensionManager,
    icon: 'i-material-symbols-bug-report-outline-rounded color-text-tertiary',
    tab: singletonTab('extensionmanager デバッグ'),
  },
  {
    kind: 'debug-extensionlog',
    pathTemplate: '/debug/extensionlog',
    component: ExtensionLog,
    icon: 'i-material-symbols-bug-report-outline-rounded color-text-tertiary',
    tab: singletonTab('extensionlog デバッグ'),
  },
  {
    kind: 'debug-denylist',
    pathTemplate: '/debug/denylist',
    component: DenyList,
    icon: 'i-material-symbols-bug-report-outline-rounded color-text-tertiary',
    tab: singletonTab('denylist デバッグ'),
  },
] as const satisfies readonly Descriptor[]
