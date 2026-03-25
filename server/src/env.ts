export interface Env {
  DB: D1Database
  IMAGES: R2Bucket
  ASSETS: Fetcher
  SESSION_SECRET: string
  SESSION_TTL_SECONDS?: string
  R2_ACCOUNT_ID: string
  R2_ACCESS_KEY_ID: string
  R2_SECRET_ACCESS_KEY: string
  R2_BUCKET_NAME: string
  R2_PRESIGN_TTL_SECONDS?: string
}

declare global {
  type AppEnv = Env
}
