import { ResponseType, fetch, Body } from "@tauri-apps/api/http";

export const scrapeSql = async (query: string, colNums: number) => {
  try {
    const formData = new FormData();
    formData.append("sql", query);
    const res = await fetch<string>(
      "https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/sql_for_erogamer_form.php",
      {
        method: "POST",
        responseType: ResponseType.Text,
        headers: {
          "content-type": "application/x-www-form-urlencoded",
        },
        body: Body.form(formData),
      }
    );
    const parser = new DOMParser();
    const doc = parser.parseFromString(res.data, "text/html");

    const rows: string[][] = [];
    doc.querySelectorAll("#query_result_main tr").forEach((tr, i) => {
      if (i === 0) {
        return;
      }
      const row: string[] = [];
      let isSkip = false;
      for (let index = 0; index < colNums; index++) {
        const scrapeIndex = index + 1;
        const col = tr.querySelector(`td:nth-child(${scrapeIndex})`);
        if (!col) {
          isSkip = true;
          break;
        }
        row.push(col.innerHTML);
      }
      if (isSkip) {
        return;
      }
      rows.push(row);
    });
    return rows;
  } catch (e) {
    console.error(e);
    return [];
  }
};
