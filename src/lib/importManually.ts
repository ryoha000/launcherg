import { commandGetExePathByLnk, commandGetGameCacheById } from "@/lib/command";
import { scrapeSql } from "@/lib/scrapeSql";
import { showErrorToast } from "@/lib/toast";

export const useImportManually = () => {
  const parseErogameScapeId = (input: string) => {
    {
      const idNumber = +input;
      if (!isNaN(idNumber)) {
        return idNumber;
      }
    }

    try {
      const url = new URL(input);
      const idString = url.searchParams.get("game");
      if (!idString) {
        return;
      }
      const idNumber = +idString;
      if (isNaN(idNumber)) {
        return;
      }
      return idNumber;
    } catch (e) {
      console.warn(e);
    }
  };

  const getNewCollectionElementByInputs = async (
    idInput: string,
    pathInput: string
  ) => {
    const id = parseErogameScapeId(idInput);
    if (!id) {
      return showErrorToast("ErogameScape の id として解釈できませんでした");
    }

    const gameCache = await commandGetGameCacheById(id);
    if (!gameCache) {
      return showErrorToast(
        "存在しない id でした。ErogameScape を確認して存在していたらバグなので @ryoha000 に連絡していただけると幸いです。"
      );
    }

    if (pathInput.toLowerCase().endsWith("exe")) {
      return { exePath: pathInput, lnkPath: null, gameCache };
    } else {
      return { exePath: null, lnkPath: pathInput, gameCache };
    }
  };

  return { getNewCollectionElementByInputs };
};
