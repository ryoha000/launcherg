import { commandGetExePathByLnk } from "@/lib/command";
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

    const gamenames = await scrapeSql(
      `select gamename from gamelist where id = ${id};`,
      1
    );
    if (gamenames.length !== 1 || gamenames[0].length !== 1) {
      showErrorToast("指定したゲームの名前が取得できませんでした");
      return;
    }
    const gamename = gamenames[0][0];

    if (pathInput.toLowerCase().endsWith("lnk")) {
      const exePath = await commandGetExePathByLnk(pathInput);
      return { id, gamename, path: exePath };
    } else {
      return { id, gamename, path: pathInput };
    }
  };

  return { getNewCollectionElementByInputs };
};
