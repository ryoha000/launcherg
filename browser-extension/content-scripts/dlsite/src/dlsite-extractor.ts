// DLsite用独立型抽出器

import { create, fromJson, toJson } from "@bufbuild/protobuf";

import {
  ExtensionRequestSchema,
  ExtensionResponseSchema,
  GameDataSchema,
  ShowNotificationRequestSchema,
  SyncGamesRequestSchema,
} from "@launcherg/shared/proto/extension_internal";

// ゲームデータのインターフェース
interface ExtractedGameData {
  store_id: string;
  title: string;
  purchase_url: string;
  purchase_date?: string;
  thumbnail_url?: string;
  additional_data: Record<string, string>;
}

// ユーティリティ関数
function generateRequestId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2);
}

// URLからstore_idを抽出するヘルパー関数
function extractStoreIdFromUrl(thumbnailUrl: string): string | null {
  // ファイル名の部分のみを対象とする
  const fileName = thumbnailUrl.split("/").pop() || "";
  const rjMatch = fileName.match(/(RJ|VJ|BJ)(\d+)/);
  if (!rjMatch) {
    return null;
  }
  return rjMatch[1] + rjMatch[2];
}

// DLsiteExtractorクラスの定義
class DLsiteExtractor {
  private isExtracting: boolean = false;
  private debugMode: boolean = true;

  constructor() {
    // 設定不要の独立型抽出器
  }

  shouldExtract(): boolean {
    // ページURL確認
    if (!window.location.hostname.includes("dlsite.com")) {
      return false;
    }

    // 新しいDLsiteのReactベースのページを検出
    const rootElement = document.getElementById("root");
    const hasLibraryContent =
      rootElement &&
      (document.querySelector("._thumbnail_1kd4u_117") !== null ||
        document.querySelector("[data-index]") !== null);

    return !!hasLibraryContent;
  }

  extractGames(): ExtractedGameData[] {
    // 新しいHTML構造に対応した直接的な抽出
    const gameContainers = document.querySelectorAll("[data-index]");
    this.debug(`Found ${gameContainers.length} potential game containers`);

    const games: ExtractedGameData[] = [];
    const seenStoreIds = new Set<string>();

    gameContainers.forEach((container, index) => {
      try {
        // 実際のゲームアイテムかどうかを確認（サムネイルがあるか）
        const thumbnailElement = container.querySelector(
          "._thumbnail_1kd4u_117 span"
        ) as HTMLElement;
        if (!thumbnailElement) {
          return;
        }

        // サムネイルURLから情報を抽出
        const bgImage = thumbnailElement.style.backgroundImage;
        const thumbnailMatch = bgImage.match(/url\("?(.+?)"\)/);
        if (!thumbnailMatch) {
          return;
        }

        const thumbnailUrl = thumbnailMatch[1];

        // URLからstore_idを抽出
        const storeId = extractStoreIdFromUrl(thumbnailUrl);
        this.debug(`Extracted store_id "${storeId}" from URL: ${thumbnailUrl}`);
        if (!storeId) {
          return;
        }

        // 重複チェック
        if (seenStoreIds.has(storeId)) {
          return;
        }
        seenStoreIds.add(storeId);

        // タイトルを抽出
        const titleElement = container.querySelector(
          "._workName_1kd4u_192 span"
        );
        const title = titleElement?.textContent?.trim() || "";

        // メーカー名を抽出
        const makerElement = container.querySelector(
          "._makerName_1kd4u_196 span"
        );
        const makerName = makerElement?.textContent?.trim() || "";

        // 購入日を抽出（親要素から探す）
        let purchaseDate = "";
        const headerElement = container
          .closest("[data-index]")
          ?.querySelector("._header_1kd4u_27 span");
        if (headerElement?.textContent?.includes("購入")) {
          purchaseDate = headerElement.textContent.replace("購入", "").trim();
        }

        // 購入URLを構築
        const purchaseUrl = `https://play.dlsite.com/maniax/work/=/product_id/${storeId}.html`;

        const gameData: ExtractedGameData = {
          store_id: storeId,
          title,
          purchase_url: purchaseUrl,
          purchase_date: purchaseDate,
          thumbnail_url: thumbnailUrl,
          additional_data: {
            maker_name: makerName,
          },
        };

        games.push(gameData);
        this.debug(`Extracted game ${index + 1}:`, gameData);
      } catch (error) {
        this.debug(`Error extracting game from container ${index}:`, error);
      }
    });

    return games;
  }

  async extractAndSync(): Promise<void> {
    if (this.isExtracting) {
      console.log("[DLsite Extractor] Already extracting, skipping");
      return;
    }

    this.isExtracting = true;

    try {
      // ページが完全に読み込まれるまで待機
      await this.waitForPageLoad();

      // ゲーム情報を抽出
      const games = this.extractGames();

      if (games.length === 0) {
        console.log("[DLsite Extractor] No games found");
        return;
      }

      console.log(`[DLsite Extractor] Found ${games.length} games`);

      // DLsite特有の処理
      const processedGames = games.map((game) => this.processDLsiteGame(game));

      // プロトバフでゲームデータを変換
      const gameDataList = processedGames.map((game) =>
        create(GameDataSchema, {
          storeId: game.store_id,
          title: game.title,
          purchaseUrl: game.purchase_url,
          purchaseDate: game.purchase_date || "",
          thumbnailUrl: game.thumbnail_url || "",
          additionalData: game.additional_data,
        })
      );

      // プロトバフメッセージを作成
      const syncRequest = create(ExtensionRequestSchema, {
        requestId: generateRequestId(),
        request: {
          case: "syncGames",
          value: create(SyncGamesRequestSchema, {
            store: "DLSite",
            games: gameDataList,
            source: "dlsite-extractor",
          }),
        },
      });

      // バックグラウンドスクリプトに送信
      chrome.runtime.sendMessage(
        toJson(ExtensionRequestSchema, syncRequest),
        (responseJson) => {
          try {
            const response = fromJson(ExtensionResponseSchema, responseJson);
            if (
              response &&
              response.success &&
              response.response.case === "syncGamesResult"
            ) {
              console.log("[DLsite Extractor] Sync successful:", response);
              this.showNotification(
                `DLsite: ${processedGames.length}個の作品を同期しました`
              );
            } else {
              console.error("[DLsite Extractor] Sync failed:", response);
              this.showNotification("DLsite: 同期に失敗しました", "error");
            }
          } catch (error) {
            console.error(
              "[DLsite Extractor] Failed to parse sync response:",
              error
            );
            this.showNotification(
              "DLsite: 同期レスポンスの解析に失敗しました",
              "error"
            );
          }
        }
      );
    } catch (error) {
      console.error("[DLsite Extractor] Extraction failed:", error);
      this.showNotification("DLsite: 作品情報の抽出に失敗しました", "error");
    } finally {
      this.isExtracting = false;
    }
  }

  private processDLsiteGame(game: ExtractedGameData): ExtractedGameData {
    // DLsiteのURLを正規化
    if (game.purchase_url && !game.purchase_url.startsWith("http")) {
      game.purchase_url = `https://play.dlsite.com${game.purchase_url}`;
    }

    // DLsite特有のstore_id処理（RJ/VJ/BJ codes）
    if (game.store_id) {
      // URLから作品コードを抽出
      const match = game.purchase_url.match(/\/(RJ|VJ|BJ)(\d+)/);
      if (match) {
        game.store_id = match[1] + match[2];
      }
      // 既存のstore_idが正しい形式かチェック
      else if (!game.store_id.match(/^(RJ|VJ|BJ)\d+$/)) {
        // 数字のみの場合はRJを付加（最も一般的）
        if (game.store_id.match(/^\d+$/)) {
          game.store_id = `RJ${game.store_id}`;
        }
      }
    }

    // サムネイルURLの正規化
    if (game.thumbnail_url) {
      if (game.thumbnail_url.startsWith("//")) {
        game.thumbnail_url = `https:${game.thumbnail_url}`;
      } else if (!game.thumbnail_url.startsWith("http")) {
        game.thumbnail_url = `https://play.dlsite.com${game.thumbnail_url}`;
      }
    }

    // 購入日の正規化（DLsiteの日付フォーマット対応）
    if (game.purchase_date) {
      game.purchase_date = this.normalizeDLsiteDate(game.purchase_date);
    }

    // タイトルのクリーンアップ（DLsiteの不要な文字列を除去）
    if (game.title) {
      game.title = this.cleanDLsiteTitle(game.title);
    }

    // DLsite特有の追加情報
    game.additional_data.store_name = "DLsite";
    game.additional_data.extraction_source = "dlsite-extractor";
    game.additional_data.extraction_timestamp = new Date().toISOString();

    // 作品の種類を判定
    if (game.store_id.startsWith("RJ")) {
      game.additional_data.work_type = "doujin";
    } else if (game.store_id.startsWith("VJ")) {
      game.additional_data.work_type = "voice";
    } else if (game.store_id.startsWith("BJ")) {
      game.additional_data.work_type = "book";
    }

    return game;
  }

  private normalizeDLsiteDate(dateStr: string): string {
    try {
      // DLsite日付フォーマット対応: "YYYY年MM月DD日", "YYYY/MM/DD", "YYYY-MM-DD"
      let cleanDate = dateStr
        .replace(/年/g, "/")
        .replace(/月/g, "/")
        .replace(/日/g, "")
        .replace(/\s+/g, "");

      const date = new Date(cleanDate);
      return date.toISOString().split("T")[0];
    } catch {
      return dateStr;
    }
  }

  private cleanDLsiteTitle(title: string): string {
    // DLsiteのタイトルから不要な情報を除去
    return title
      .replace(/\[.*?\]/g, "") // [サークル名] などを除去
      .replace(/（.*?）/g, "") // 全角括弧の内容を除去
      .replace(/\(.*?\)/g, "") // 半角括弧の内容を除去
      .replace(/\s+/g, " ") // 連続する空白を単一の空白に
      .trim();
  }

  private async waitForPageLoad(): Promise<void> {
    return new Promise((resolve) => {
      if (document.readyState === "complete") {
        // DLsiteは動的コンテンツが多いので少し長めに待機
        setTimeout(resolve, 2000);
      } else {
        window.addEventListener("load", () => {
          setTimeout(resolve, 2000);
        });
      }
    });
  }

  private showNotification(
    message: string,
    type: "success" | "error" = "success"
  ): void {
    // プロトバフで通知メッセージを作成
    const notificationRequest = create(ExtensionRequestSchema, {
      requestId: generateRequestId(),
      request: {
        case: "showNotification",
        value: create(ShowNotificationRequestSchema, {
          title: "Launcherg DL Store Sync",
          message,
          iconType: type,
        }),
      },
    });

    // ブラウザ通知を表示
    chrome.runtime.sendMessage(
      toJson(ExtensionRequestSchema, notificationRequest)
    );

    // ページ内通知も表示
    this.showInPageNotification(message, type);
  }

  private showInPageNotification(
    message: string,
    type: "success" | "error"
  ): void {
    const notification = document.createElement("div");
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: ${type === "success" ? "#4CAF50" : "#f44336"};
      color: white;
      padding: 12px 20px;
      border-radius: 4px;
      z-index: 10000;
      font-family: Arial, sans-serif;
      font-size: 14px;
      box-shadow: 0 2px 10px rgba(0,0,0,0.3);
      animation: slideIn 0.3s ease-out;
    `;

    notification.textContent = message;
    document.body.appendChild(notification);

    // 4秒後に自動削除
    setTimeout(() => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification);
      }
    }, 4000);
  }

  private debug(message: string, ...args: any[]): void {
    if (this.debugMode) {
      console.log(`[DLsite Extractor] ${message}`, ...args);
    }
  }
}

// グローバル変数とヘルパー関数
let currentUrl = window.location.href;

function initDLsiteExtractor() {
  const extractor = new DLsiteExtractor();

  if (extractor.shouldExtract()) {
    console.log(
      "[DLsite Extractor] Target page detected - Starting extraction on DLsite"
    );
    extractor.extractAndSync();
  } else {
    console.log("[DLsite Extractor] Not a target page - skipping extraction");
  }
}

// CSS animation
const style = document.createElement("style");
style.textContent = `
  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
`;
document.head.appendChild(style);

// ページ変更の監視（SPA対応）
const observer = new MutationObserver(() => {
  if (window.location.href !== currentUrl) {
    currentUrl = window.location.href;
    // URL変更時に再度チェック
    setTimeout(() => {
      initDLsiteExtractor();
    }, 2000);
  }
});

observer.observe(document.body, {
  childList: true,
  subtree: true,
});

// 初期化処理
console.log("[DLsite Extractor] Script loaded");

// 即座に抽出を開始（設定不要）
setTimeout(() => {
  initDLsiteExtractor();
}, 1000);
