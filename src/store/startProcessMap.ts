import { createWritable } from "@/lib/utils";

export const [startProcessMap, getStartProcessMap] = createWritable<{
  [key: string]: number;
}>({});
