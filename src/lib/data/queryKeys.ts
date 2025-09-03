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
}
