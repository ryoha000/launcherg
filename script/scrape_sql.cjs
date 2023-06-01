const STEP = 5000;
const MAX_SCRAPE_COUNT = 20;

const fs = require("fs");
const { parse } = require("node-html-parser");

const scrape = async (idCursor) => {
  try {
    const formData = new FormData();
    if (typeof idCursor !== "number" || isNaN(idCursor)) {
      return [];
    }
    const query = `SELECT id, gamename FROM gamelist WHERE id >= ${idCursor} AND id < ${
      idCursor + STEP
    };`;
    formData.append("sql", query);
    const res = await fetch(
      "https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/sql_for_erogamer_form.php",
      {
        method: "POST",
        body: formData,
      }
    );
    const text = await res.text();
    const dom = parse(text);

    const games = [];
    dom.querySelectorAll("#query_result_main tr").forEach((tr, i) => {
      if (i === 0) {
        return;
      }
      const id = tr.querySelector("td:nth-child(1)");
      const gamename = tr.querySelector("td:nth-child(2)");
      if (!id || !gamename) return;
      games.push({
        id: +id.innerHTML,
        gamename: gamename.innerHTML,
      });
    });
    return games;
  } catch (e) {
    console.error(e);
    return [];
  }
};

const save = (data) => {
  const jsonData = JSON.stringify(data);

  fs.writeFile("all_games.json", jsonData, "utf8", (err) => {
    if (err) {
      console.error("An error occurred while writing to the file:", err);
      return;
    }
    console.log("The file has been saved!");
  });
};

const execute = async () => {
  let idCursor = 0;
  const games = [];
  for (let i = 0; i < MAX_SCRAPE_COUNT; i++) {
    const appendGames = await scrape(idCursor);
    if (!appendGames.length) {
      console.log(`end within ${i + 1} loop. games.length: ${games.length}`);
      break;
    }
    games.push(...appendGames);
    await new Promise((resolve) => setTimeout(resolve, 5000));
    idCursor += STEP;
  }

  save(games);
};

execute();
