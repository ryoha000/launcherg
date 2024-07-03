import {
  SkyWayContext,
  SkyWayRoom,
  SkyWayStreamFactory,
  uuidV4,
  type RoomPublication,
  type LocalStream,
  type LocalDataStream,
  type RemoteRoomMember,
} from "@skyway-sdk/room";
import { memo } from "../store/memo";
import { createWritable } from "@/lib/utils";
import { fetch } from "@tauri-apps/plugin-http";
import { readFile } from "@tauri-apps/plugin-fs";
import { commandSaveScreenshotByPid } from "@/lib/command";
import { getStartProcessMap } from "@/store/startProcessMap";
import { showErrorToast } from "@/lib/toast";
import { useChunk } from "@/lib/chunk";

type PingMessage = { type: "ping" };
type MemoMessage = {
  type: "memo";
  text: string;
  gameId: number;
};
type InitMessage = { type: "init"; gameId: number };
type InitResponseMessage = {
  type: "init_response";
  gameId: number;
  initialMemo: MemoMessage;
};
type TakeScreenshotMessage = {
  type: "take_screenshot";
  gameId: number;
  cursorLine: number;
};
type ImageMetadataMessage = {
  type: "image_metadata";
  path: string;
  key: number;
  totalChunkLength: number;
  mimeType: string;
};

type LocalMessage =
  | PingMessage
  | MemoMessage
  | InitResponseMessage
  | ImageMetadataMessage;
type RemoteMessage =
  | PingMessage
  | MemoMessage
  | InitMessage
  | TakeScreenshotMessage;

const createSkyWay = () => {
  const roomId = uuidV4();
  const sentImagePathSet = new Set<string>();
  const { createChunks } = useChunk();

  const sendImagesAsChunks = async (imagePaths: string[]) => {
    await Promise.all(
      imagePaths.map((path) => {
        return new Promise<void>(async (resolve) => {
          const [{ chunkId, mimeType, totalChunkLength }, chunks] =
            await createChunks(path);
          const message: ImageMetadataMessage = {
            type: "image_metadata",
            path,
            key: chunkId,
            totalChunkLength,
            mimeType,
          };
          sendMessage(message);
          chunks.forEach(sendBinaryMessage);
          resolve();
        });
      })
    );
  };

  const getMemoImagePaths = (text: string) => {
    const regex = /!\[.*?\]\((.*?)\)/g;
    const paths: string[] = [];
    let match: RegExpExecArray | null = null;
    while ((match = regex.exec(text)) !== null) {
      paths.push(match[1]);
    }

    return paths;
  };
  const getMemo = (gameId: number): { value: string; imagePaths: string[] } => {
    const memoKey = `smde_memo-${gameId}`;
    const memo = localStorage.getItem(memoKey) ?? "";

    const paths = getMemoImagePaths(memo);

    return { value: memo, imagePaths: paths };
  };
  const setRemoteMemo = (workId: number, text: string) => {
    const memoKey = `smde_memo-${workId}`;
    localStorage.setItem(memoKey, text);

    memo.update((v) => {
      // 開いてないときはわざわざ store に入れない
      if (!v.find((v) => v.workId === workId)) {
        return v;
      }
      return v.map((v) =>
        v.workId === workId ? { ...v, value: text, lastModified: "remote" } : v
      );
    });
  };

  const createInitResponseMessage = async (workId: number) => {
    const { value, imagePaths } = getMemo(workId);
    imagePaths.forEach((path) => sentImagePathSet.add(path));
    await sendImagesAsChunks(imagePaths);

    const message: InitResponseMessage = {
      type: "init_response",
      gameId: workId,
      initialMemo: {
        type: "memo",
        text: value,
        gameId: workId,
      },
    };
    return message;
  };

  const cleanupFuncs: (() => void)[] = [];
  const cleanup = () => {
    cleanupFuncs.forEach((func) => func());
  };

  let dataStream: LocalDataStream | undefined = undefined;
  const setDataStream = async () => {
    const response = await fetch("https://launcherg.ryoha.moe/connect", {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
    });
    const { authToken } = (await response.json()) as { authToken: string };

    const context = await SkyWayContext.Create(authToken);
    const room = await SkyWayRoom.FindOrCreate(context, {
      type: "p2p",
      name: roomId,
    });
    const me = await room.join();
    me.onFatalError.add(() => {
      dataStream = undefined;
      cleanup();
      showErrorToast("接続が切断されました。");
    });

    const onPublicate = async (publication: RoomPublication<LocalStream>) => {
      if (publication.publisher.id === me.id) return;
      if (publication.contentType !== "data") return;

      const { stream } = await me.subscribe(publication.id);
      if (stream.contentType !== "data") return;

      const { removeListener } = stream.onData.add(async (data) => {
        if (typeof data !== "string") return;

        const message: RemoteMessage = JSON.parse(data);
        if (message.type !== "ping") {
          console.log("receive message", message);
        }
        switch (message.type) {
          case "ping":
            return;
          case "memo":
            setRemoteMemo(message.gameId, message.text);
            return;
          case "init":
            const response = await createInitResponseMessage(message.gameId);
            sendMessage(response);
            break;
          case "take_screenshot":
            try {
              const imagePath = await commandSaveScreenshotByPid(
                message.gameId,
                getStartProcessMap()[message.gameId]
              );
              const prev = getMemo(message.gameId).value;
              const lines = prev.split("\n");
              const newLines: string[] = [];
              for (let i = 0; i < lines.length; i++) {
                newLines.push(lines[i]);
                if (i === message.cursorLine) {
                  newLines.push(`![](${imagePath})`);
                  newLines.push("");
                }
              }
              const newMemo = newLines.join("\n");
              setRemoteMemo(message.gameId, newMemo);
              syncMemo(message.gameId, newMemo);
            } catch (e) {
              showErrorToast("スクリーンショットの取得に失敗しました。");
              console.error(e);
            }
        }
      });
      cleanupFuncs.push(removeListener);

      // PC側の準備が完了したら subscribe させる
      await (publication.publisher as RemoteRoomMember).subscribe(
        myPublication.id
      );
    };

    dataStream = await SkyWayStreamFactory.createDataStream();
    const myPublication = await me.publish(dataStream);

    const pingTimer = setInterval(() => {
      if (!dataStream) return;
      const message: PingMessage = { type: "ping" };
      sendMessage(message);
    }, 10000);
    cleanupFuncs.push(() => clearInterval(pingTimer));

    room.publications.forEach(onPublicate);
    room.onStreamPublished.add((e) => onPublicate(e.publication));
  };

  const connect = async (workId: number, seiyaUrl: string) => {
    if (!dataStream) {
      await setDataStream();
    }
    const url = new URL("https://launcherg.ryoha.moe");
    url.searchParams.set("seiyaUrl", seiyaUrl);
    url.searchParams.set("roomId", roomId);
    url.searchParams.set("gameId", workId.toString());
    // return `http://127.0.0.1:8788?seiyaUrl=${seiyaUrl}&roomId=${roomId}&gameId=${workId}`;
    return url.toString();
  };

  const sendMessage = (message: LocalMessage) => {
    if (!dataStream) return;
    if (message.type !== "ping") {
      console.log("send message", message);
    }

    dataStream.write(JSON.stringify(message));
  };
  const sendBinaryMessage = (message: Uint8Array) => {
    if (!dataStream) return;

    dataStream.write(message);
  };

  const syncMemo = async (workId: number, text: string) => {
    if (!dataStream) return;
    const imagePaths = getMemoImagePaths(text);
    const notSharedImages = imagePaths.filter(
      (path) => !sentImagePathSet.has(path)
    );
    await sendImagesAsChunks(notSharedImages);

    const message: MemoMessage = {
      type: "memo",
      text,
      gameId: workId,
    };
    sendMessage(message);
  };

  return { connect, syncMemo, cleanup, roomId };
};

export const skyWay = createSkyWay();
