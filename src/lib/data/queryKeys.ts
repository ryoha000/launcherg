export const queryKeys = {
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
  storagePaths: {
    all: () => ['storagePaths'] as const,
  },
}
