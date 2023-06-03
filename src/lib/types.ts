export type Work = {
  id: number;
  name: string;
  brandId: number;
  brandName: string;
  officialHomePage: string;
  sellday: string;
  imgUrl: string;
  statistics: Statistics;
  creators: Creators;
};

export type Statistics = {
  median: number;
  average: number;
  count: number;
};

export type Creators = {
  illustrators: Creator[];
  writers: Creator[];
  voiceActors: VoiceActor[];
};

export type Creator = {
  id: number;
  name: string;
};

export const VoiceActorImportance = {
  Main: 0,
  Sub: 1,
  Mob: 2,
} as const;

export type VoiceActor = {
  role: string;
  importance: (typeof VoiceActorImportance)[keyof typeof VoiceActorImportance];
} & Creator;

export type Collection = {
  id: number;
  name: string;
};

export type CollectionElement = {
  id: number; // Work.id と同じ
  gamename: string;
  path: string;
  icon: string;
};
