import type { SeiyaDataPair } from "@/lib/types";
import { ResponseType, fetch } from "@tauri-apps/api/http";
import Encoding from "encoding-japanese";

export const getSeiyaDataPairs = async () => {
  const response = await fetch<Uint16Array>(
    "https://seiya-saiga.com/game/kouryaku.html",
    {
      method: "GET",
      responseType: ResponseType.Binary,
    }
  );
  const parser = new DOMParser();
  const htmlUnicodeArray = Encoding.convert(response.data, {
    to: "UNICODE",
    from: "SJIS",
  });
  const html = Encoding.codeToString(htmlUnicodeArray);
  const doc = parser.parseFromString(html, "text/html");

  const trs = doc.getElementsByTagName("tr");

  const pairs: SeiyaDataPair[] = [];
  for (const tr of trs) {
    const atag = tr.getElementsByTagName("a")?.[0];
    if (!atag) {
      continue;
    }

    const name = atag.innerHTML;
    const path = atag.getAttribute("href");
    if (!name || !path) {
      continue;
    }

    let url = "";
    if (path?.startsWith("http")) {
      url = path;
    } else {
      url = `https://seiya-saiga.com/game/${path}`;
    }

    pairs.push([name, url]);
  }
  return pairs;
};
