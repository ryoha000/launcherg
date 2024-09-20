import type { SeiyaDataPair } from "@/lib/types";
import { fetch } from "@tauri-apps/plugin-http";
import Encoding from "encoding-japanese";

export const getSeiyaDataPairs = async () => {
  const response = await fetch("https://seiya-saiga.com/game/kouryaku.html", {
    method: "GET",
  });
  const data = await response.arrayBuffer();
  const uint8array = new Uint8Array(data);
  const parser = new DOMParser();
  const htmlUnicodeArray = Encoding.convert(uint8array, {
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
