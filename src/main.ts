import * as Sentry from '@sentry/svelte'
import { mount } from 'svelte'
import App from './App.svelte'
import 'virtual:uno.css'
import '@unocss/reset/tailwind-compat.css'
import './index.scss'
import 'tippy.js/dist/tippy.css'
import 'simplebar'
import 'simplebar/dist/simplebar.min.css'
import 'easymde/dist/easymde.min.css'
import './toast.scss'

if (import.meta.env.PROD) {
  Sentry.init({
    dsn: 'https://0d1230d42a9e11c8f3ca34fb206c841a@o4509659869151232.ingest.us.sentry.io/4509659880357888',
    // Setting this option to true will send default PII data to Sentry.
    // For example, automatic IP address collection on events
    sendDefaultPii: true,
  })
}

const app = mount(App, { target: document.getElementById('app')! })

export default app
