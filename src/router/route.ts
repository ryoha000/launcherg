import type { RouteConfig } from '@mateothegreat/svelte5-router'
import DenyList from '@/views/Debug/DenyList.svelte'
import DmmPacks from '@/views/Debug/DmmPacks.svelte'
import ExtensionLog from '@/views/Debug/ExtensionLog.svelte'
import ExtensionManager from '@/views/Debug/ExtensionManager.svelte'
import ProcTailDebug from '@/views/Debug/ProcTail.svelte'
import Home from '@/views/Home.svelte'
import Memo from '@/views/Memo.svelte'
import Settings from '@/views/Settings.svelte'
import Work from '@/views/Work.svelte'

export const routes: RouteConfig[] = [
  { path: '/', component: Home },
  { path: '/works/(?<id>.*)', component: Work },
  { path: '/memos/(?<id>.*)', component: Memo },
  { path: '/settings', component: Settings },
  { path: '/debug/extensionmanager', component: ExtensionManager },
  { path: '/debug/extensionlog', component: ExtensionLog },
  { path: '/debug/denylist', component: DenyList },
  { path: '/debug/dmmpacks', component: DmmPacks },
  { path: '/debug/proctail', component: ProcTailDebug },
  // TODO: 404
]
