import { defineConfig } from 'drizzle-kit'

const accountId = process.env.CLOUDFLARE_ACCOUNT_ID ?? ''
const databaseId = process.env.CLOUDFLARE_D1_DATABASE_ID ?? ''
const token = process.env.CLOUDFLARE_API_TOKEN ?? ''

export default defineConfig({
  dialect: 'sqlite',
  driver: 'd1-http',
  schema: './src/db/schema.ts',
  out: './migrations',
  dbCredentials: {
    accountId,
    databaseId,
    token,
  },
})
