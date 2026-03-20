export const queryKeys = {
  denyList: {
    all: () => ['denyList'] as const,
  },
  dmmPack: {
    all: () => ['dmmPack'] as const,
  },
  workDetails: {
    all: () => ['workDetails'] as const,
    byId: (id: string) => ['workDetails', id] as const,
  },
  workLnk: {
    all: () => ['workLnk'] as const,
    byId: (id: string) => ['workLnk', id] as const,
  },
  imageQueue: {
    unfinished: () => ['imageQueue', 'unfinished'] as const,
    finished: () => ['imageQueue', 'finished'] as const,
  },
}
