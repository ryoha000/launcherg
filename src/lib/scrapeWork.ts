import type { Creator, VoiceActor, Work } from "@/lib/types";
import { convertSpecialCharacters } from "@/lib/utils";
import { fetch } from "@tauri-apps/plugin-http";

const BASE_REQUEST_PATH =
  "https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki";

const getCreator = (elm: HTMLElement) => {
  const creators: Creator[] = [];
  const aTags = elm.getElementsByTagName("a");
  for (let i = 0; i < aTags.length; i++) {
    const aTag = aTags[i];
    const creator: Creator = {
      id: +(
        aTag.getAttribute("href")?.replace("creater.php?creater=", "") ?? "0"
      ),
      name: convertSpecialCharacters(aTag.innerHTML),
    };
    creators.push(creator);
  }
  return creators;
};
const getVoiceActors = (elm: HTMLElement) => {
  const creators: VoiceActor[] = [];
  const aTags = elm.getElementsByTagName("a");
  const spanTags = elm.getElementsByTagName("span");
  for (let i = 0; i < aTags.length; i++) {
    const aTag = aTags[i];
    const creator: Creator = {
      id: +(
        aTag.getAttribute("href")?.replace("creater.php?creater=", "") ?? "0"
      ),
      name: convertSpecialCharacters(aTag.innerHTML),
    };
    if (spanTags.length > i) {
      const color = spanTags[i].getAttribute("style");
      const voiceActor: VoiceActor = {
        ...creator,
        role: convertSpecialCharacters(spanTags[i].innerHTML),
        importance: color?.includes("bold")
          ? 0
          : color?.includes("black")
          ? 1
          : 2,
      };
      creators.push(voiceActor);
    }
  }
  return creators;
};
const getMusics = (elements: HTMLCollectionOf<HTMLTableCellElement>) => {
  const musics: string[] = [];
  for (const elm of elements) {
    const aTag = elm.getElementsByTagName("a")[0];
    if (!aTag) {
      continue;
    }
    if (!aTag.href.includes("music.php?music=")) {
      continue;
    }
    musics.push(aTag.innerHTML);
  }
  return musics;
};

export const getWorkByScrape = async (id: number) => {
  const response = await fetch(`${BASE_REQUEST_PATH}/game.php?game=${id}`, {
    method: "GET",
  });
  const parser = new DOMParser();
  const doc = parser.parseFromString(await response.text(), "text/html");

  const gameTitle = doc.getElementById("game_title");
  const softTitle = doc.getElementById("soft-title");
  const illustrators = doc
    .getElementById("genga")
    ?.getElementsByTagName("td")[0];
  const writers = doc.getElementById("shinario")?.getElementsByTagName("td")[0];
  const voiceActors = doc
    .getElementById("seiyu")
    ?.getElementsByTagName("td")[0];

  const musics = doc
    .getElementById("music_summary_main")
    ?.getElementsByTagName("td");
  const work: Work = {
    id: id,
    name: convertSpecialCharacters(
      gameTitle?.getElementsByTagName("a")[0].innerHTML ?? ""
    ),
    brandId: +(
      softTitle
        ?.getElementsByTagName("a")[0]
        ?.getAttribute("href")
        ?.replace("brand.php?brand=", "") ?? "0"
    ),
    brandName: convertSpecialCharacters(
      softTitle?.getElementsByTagName("a")[0].innerHTML ?? ""
    ),
    sellday: softTitle?.getElementsByTagName("a")[1].innerHTML ?? "2030-01-01",
    imgUrl:
      doc.getElementById("main_image")?.getElementsByTagName("img")[0].src ??
      "",
    officialHomePage:
      gameTitle?.getElementsByTagName("a")[0].getAttribute("href") ?? "",
    statistics: {
      median: +(
        doc.getElementById("median")?.getElementsByTagName("td")[0].innerHTML ??
        "0"
      ),
      count: +(
        doc.getElementById("count")?.getElementsByTagName("td")[0].innerHTML ??
        "0"
      ),
      average: +(
        doc.getElementById("average")?.getElementsByTagName("td")[0]
          .innerHTML ?? "0"
      ),
      playTime:
        doc.getElementById("play_time")?.getElementsByTagName("td")[0]
          .innerHTML ?? "0時間",
    },
    creators: {
      illustrators: illustrators ? getCreator(illustrators) : [],
      writers: writers ? getCreator(writers) : [],
      voiceActors: voiceActors ? getVoiceActors(voiceActors) : [],
    },
    musics: musics ? getMusics(musics) : [],
  };
  return work;
};
