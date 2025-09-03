import { works } from '@/store/works'

function createWorkImages() {
  return {
    async get(id: number): Promise<string | undefined> {
      const work = await works.get(id)
      return work?.imgUrl || undefined
    },
  }
}

export const workImages = createWorkImages()
