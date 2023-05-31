export type Work = {
  id: number;
  name: string;
  furigana: string;
  brandId: number;
  brandName: string;
  officialHomePage: string;
  sellday: string;
  imgUrl: string;
  median: number;
  average: number;
  count: number;
  gengas: Creator[];
  sinarios: Creator[];
  seiyus: Creator[];
  createdAt: Date;
};

export type Creator = {
  id: number;
  name: string;
};

export type Collection = {
  id: string;
  name: string;
  elements: CollectionElement[];
};

export type CollectionElement = {
  id: number; // Work.id と同じ
  gamename: string;
  path: string;
  iconPath: string;
};
