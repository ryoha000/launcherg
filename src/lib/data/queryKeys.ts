export const queryKeys = {
  denyList: {
    all: () => ['denyList'] as const,
  },
  dmmPack: {
    all: () => ['dmmPack'] as const,
  },
  workDetails: {
    all: () => ['workDetails'] as const,
    byId: (id: number) => ['workDetails', id] as const,
  },
  imageQueue: {
    unfinished: () => ['imageQueue', 'unfinished'] as const,
    finished: () => ['imageQueue', 'finished'] as const,
  },
}
