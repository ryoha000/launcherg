import {
  SkyWayContext,
  SkyWayRoom,
  SkyWayStreamFactory,
  uuidV4,
  type RoomPublication,
  type LocalStream,
  type LocalDataStream,
} from "@skyway-sdk/room";
import { memo } from "../store/memo";
import { createWritable } from "@/lib/utils";
import { ResponseType, fetch } from "@tauri-apps/api/http";
import { readBinaryFile } from "@tauri-apps/api/fs";

type PingMessage = { type: "ping" };
type MemoMessage = {
  type: "memo";
  text: string;
  gameId: number;
  base64Images: { path: string; base64: string }[];
};
type InitMessage = { type: "init"; gameId: number; memberId: string };
type InitResponseMessage = {
  type: "init_response";
  gameId: number;
  initialMemo: MemoMessage;
};

type LocalMessage = PingMessage | MemoMessage | InitResponseMessage;
type RemoteMessage = PingMessage | MemoMessage | InitMessage;

const createSkyWay = () => {
  const roomId = uuidV4();
  const [base64ImagesStore, getBase64Images] = createWritable<
    {
      path: string;
      dataUrl: string;
    }[]
  >([]);

  const getBase64Image = async (filePath: string) => {
    // ファイルをバイナリとして読み込む
    const data = await readBinaryFile(filePath);
    const lowerCasePath = filePath.toLowerCase();
    // MIME タイプを推定 (ここでは ".png" の場合 "image/png" としていますが、他の形式もサポートする場合は調整が必要)
    const mimeType = (function () {
      if (lowerCasePath.endsWith(".png")) return "image/png";
      if (lowerCasePath.endsWith(".jpg") || lowerCasePath.endsWith(".jpeg"))
        return "image/jpeg";
      if (lowerCasePath.endsWith(".gif")) return "image/gif";
      if (lowerCasePath.endsWith(".webp")) return "image/webp";
      throw new Error("Unsupported file type");
    })();
    // DataURL を生成
    let binary = "";
    for (let i = 0; i < data.byteLength; i++) {
      binary += String.fromCharCode(data[i]);
    }
    const base64 = btoa(binary);
    return { dataUrl: `data:${mimeType};base64,${base64}`, path: filePath };
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
      if (v.find((v) => v.workId !== workId)) {
        return v;
      }
      return v.map((v) =>
        v.workId === workId ? { ...v, value: text, lastModified: "remote" } : v
      );
    });
  };

  const createInitResponseMessage = async (workId: number) => {
    const { value, imagePaths } = getMemo(workId);
    const images = await Promise.all(imagePaths.map(getBase64Image));
    base64ImagesStore.update((current) => [
      ...current,
      ...images.map((v) => ({
        path: v.path,
        dataUrl: v.dataUrl,
      })),
    ]);

    const message: InitResponseMessage = {
      type: "init_response",
      gameId: workId,
      initialMemo: {
        type: "memo",
        text: value,
        gameId: workId,
        base64Images: images.map((v) => ({
          path: v.path,
          base64: v.dataUrl,
        })),
      },
    };
    return message;
  };

  const cleanupFuncs: (() => void)[] = [];
  const cleanup = () => {
    cleanupFuncs.forEach((func) => func());
  };

  let dataStream: LocalDataStream | undefined = undefined;
  const sentInitResponseSet = new Set<string>();
  const setDataStream = async () => {
    const response = await fetch<{ authToken: string }>(
      "https://launcherg.ryoha.moe/connect",
      {
        method: "POST",
        responseType: ResponseType.JSON,
        headers: {
          "content-type": "application/json",
        },
      }
    );
    const authToken = response.data.authToken;

    const context = await SkyWayContext.Create(authToken);
    const room = await SkyWayRoom.FindOrCreate(context, {
      type: "p2p",
      name: roomId,
    });
    const me = await room.join();
    me.onFatalError.add(() => {
      dataStream = undefined;
      cleanup();
      alert("接続が切断されました。");
    });

    const onPublicate = async (publication: RoomPublication<LocalStream>) => {
      if (publication.publisher.id === me.id) return;
      if (publication.contentType !== "data") return;

      const { stream } = await me.subscribe(publication.id);
      if (stream.contentType !== "data") return;

      const { removeListener } = stream.onData.add(async (data) => {
        if (typeof data !== "string") return;

        const message: RemoteMessage = JSON.parse(data);
        switch (message.type) {
          case "ping":
            return;
          case "memo":
            setRemoteMemo(message.gameId, message.text);
            return;
          case "init":
            if (sentInitResponseSet.has(message.memberId)) return;
            const response = await createInitResponseMessage(message.gameId);
            sentInitResponseSet.add(message.memberId);
            const member = room.members.find((v) => v.id === message.memberId);
            if (!member) {
              alert(
                "初期化リクエストを送信したクライアント(QRコードを読んだ端末)が見つかりません。"
              );
              throw new Error("member not found");
            }
            await member.subscribe(myPublication.id);
            sendMessage(response);
            break;
        }
      });
      cleanupFuncs.push(removeListener);
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
    return `http://127.0.0.1:8788?seiyaUrl=${seiyaUrl}&roomId=${roomId}&gameId=${workId}`;
    // return `https://launcherg.ryoha.moe?seiyaUrl=${seiyaUrl}&roomId=${roomId}&gameId=${workId}`;
  };

  const sendMessage = (message: LocalMessage) => {
    if (!dataStream) return;

    dataStream.write(JSON.stringify(message));
  };

  const syncMemo = async (workId: number, text: string) => {
    if (!dataStream) return;
    const imagePaths = getMemoImagePaths(text);
    const notSharedImages = imagePaths.filter(
      (path) =>
        getBase64Images().findIndex((image) => image.path === path) === -1
    );
    const images = await Promise.all(
      notSharedImages.map((path) => getBase64Image(path))
    );
    base64ImagesStore.update((current) => [
      ...current,
      ...images.map((v) => ({
        path: v.path,
        dataUrl: v.dataUrl,
      })),
    ]);

    const message: MemoMessage = {
      type: "memo",
      text,
      gameId: workId,
      base64Images: images.map((v) => ({
        path: v.path,
        base64: v.dataUrl,
      })),
    };
    sendMessage(message);
  };

  return { connect, syncMemo, cleanup, roomId };
};

export const skyWay = createSkyWay();
