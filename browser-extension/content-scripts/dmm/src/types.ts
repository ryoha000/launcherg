export interface DmmExtractedGame {
  storeId: string
  category: string
  subcategory: string
  title: string
  imageUrl: string
  parentPack?: {
    storeId: string
    category: string
    subcategory: string
  }
}
