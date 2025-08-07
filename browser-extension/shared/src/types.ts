// 共通の型定義

export interface ExtractedGameData {
  store_id: string
  title: string
  purchase_url: string
  purchase_date?: string
  thumbnail_url?: string
  additional_data: Record<string, string>
}
