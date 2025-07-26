import type { RouteConfig } from '@mateothegreat/svelte5-router'
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
  { path: '/debug/proctail', component: ProcTailDebug },
  // TODO: 404
]
