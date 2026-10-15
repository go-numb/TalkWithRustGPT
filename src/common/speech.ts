// AI 回答の読み上げ: テキスト整形・分割と、音声生成→再生の順次実行を担う。
// DOM や Web Audio に依存する部分は呼び出し側から注入し、Node のテストで検証できるようにする。

export type TtsVoice = "female" | "male";

export const TTS_VOICES: readonly { label: string; value: TtsVoice }[] = [
  { label: "女性の声", value: "female" },
  { label: "男性の声", value: "male" },
];

export const isTtsVoice = (value: unknown): value is TtsVoice =>
  value === "female" || value === "male";

// Rust 側 create_response が回答末尾に付けるメタ情報。読み上げない。
const RESPONSE_FOOTER = /\s*Model: [^\n]*, Total token: \d+, Elaps: -?\d+s\s*$/;

export function toSpeechText(text: string): string {
  return text
    .replace(RESPONSE_FOOTER, "")
    .replace(/[ \t ]+/g, " ")
    .replace(/ *\n */g, "\n")
    .replace(/\n{2,}/g, "\n")
    .trim();
}

// 文末で区切る。英文のピリオドは直後が空白のときだけ文末とみなす (3.14 などを割らない)。
const SENTENCE = /[^。！？!?\n]*?(?:[。！？!?]+|\.(?=\s)|\n|$)/g;

/**
 * 読み上げ用に文単位でまとめて分割する。
 * 最初のまとまりは短くして再生開始を早め、以降は maxChars までまとめてリクエスト数を抑える。
 */
export function splitForSpeech(text: string, maxChars = 400, firstMaxChars = 120): string[] {
  const chunks: string[] = [];
  let current = "";
  const limit = () => (chunks.length === 0 ? firstMaxChars : maxChars);
  const flush = () => {
    const chunk = current.trim();
    if (chunk) chunks.push(chunk);
    current = "";
  };

  for (const sentence of text.match(SENTENCE) ?? []) {
    if (!sentence.trim()) {
      // 句点直後の改行など。区切りの空白は残し、まとまりの端では flush() で落とす。
      current += sentence;
      continue;
    }
    if (current.trim() && (current + sentence).trim().length > limit()) flush();
    current += sentence;
    // 句点のない長文は文字数で切る (サロゲートペアは割らない)。
    while (current.trim().length > limit()) {
      const chars = Array.from(current.trimStart());
      const size = limit();
      current = chars.slice(0, size).join("");
      flush();
      current = chars.slice(size).join("");
    }
  }
  flush();
  return chunks;
}

/** Tauri の IPC は通常 ArrayBuffer を返すが、postMessage へのフォールバック時は数値配列で届く。 */
export function toArrayBuffer(data: ArrayBuffer | ArrayLike<number>): ArrayBuffer {
  if (data instanceof ArrayBuffer) return data;
  return Uint8Array.from(data).buffer as ArrayBuffer;
}

export type Synthesize = (text: string) => Promise<ArrayBuffer>;
export type PlayAudio = (audio: ArrayBuffer, signal: AbortSignal) => Promise<void>;

/**
 * 分割したテキストを順に音声化して再生する。再生中に次のまとまりを先読みする。
 * 新しい speak() や stop() で前の読み上げは中断され、false で解決する。
 */
export class SpeechSession {
  private controller: AbortController | null = null;

  get active(): boolean {
    return this.controller !== null;
  }

  async speak(
    chunks: readonly string[],
    synthesize: Synthesize,
    playAudio: PlayAudio,
    onPlay?: () => void,
  ): Promise<boolean> {
    this.stop();
    const controller = new AbortController();
    this.controller = controller;
    const { signal } = controller;
    const aborted = new Promise<null>((resolve) => {
      signal.addEventListener("abort", () => resolve(null), { once: true });
    });

    try {
      let pending: Promise<ArrayBuffer> | null = chunks.length ? synthesize(chunks[0]) : null;
      for (let index = 0; pending; index++) {
        const audio = await Promise.race([pending, aborted]);
        if (audio === null || signal.aborted) return false;

        pending = index + 1 < chunks.length ? synthesize(chunks[index + 1]) : null;
        // 中断後に届いた先読みの失敗を未処理の reject にしない。
        pending?.catch(() => undefined);

        onPlay?.();
        await playAudio(audio, signal);
        if (signal.aborted) return false;
      }
      return true;
    } catch (err) {
      if (signal.aborted) return false;
      throw err;
    } finally {
      if (this.controller === controller) this.controller = null;
    }
  }

  stop(): void {
    this.controller?.abort();
    this.controller = null;
  }
}

/**
 * Web Audio で WAV を再生する。signal の中断で即座に停止する。
 * 自動再生制限を避けるため、unlock() はクリック処理の中で同期的に呼ぶ。
 */
export function createWebAudioPlayer(): { unlock: () => void; play: PlayAudio } {
  let context: AudioContext | null = null;
  const unlock = () => {
    context ??= new AudioContext();
    if (context.state === "suspended") void context.resume();
  };
  const play: PlayAudio = async (audio, signal) => {
    unlock();
    const ctx = context as AudioContext;
    if (ctx.state === "suspended") await ctx.resume();
    const buffer = await ctx.decodeAudioData(audio);
    if (signal.aborted) return;

    await new Promise<void>((resolve) => {
      const source = ctx.createBufferSource();
      source.buffer = buffer;
      source.connect(ctx.destination);
      const stop = () => source.stop();
      source.onended = () => {
        signal.removeEventListener("abort", stop);
        source.disconnect();
        resolve();
      };
      signal.addEventListener("abort", stop, { once: true });
      source.start();
    });
  };
  return { unlock, play };
}

/** 表示中の回答 HTML から読み上げ用テキストを作る。コードは読まず、数式は TeX を読む。 */
export function extractSpeechText(root: HTMLElement): string {
  const clone = root.cloneNode(true) as HTMLElement;
  clone.querySelectorAll(".katex").forEach((node) => {
    const tex = node.querySelector('annotation[encoding="application/x-tex"]')?.textContent;
    node.replaceWith(tex ?? node.textContent ?? "");
  });
  clone.querySelectorAll("pre").forEach((node) => node.replaceWith("（コードは省略します）\n"));
  // 切り離した要素の innerText は改行を返さないため、ブロック要素の後に改行を補う。
  clone.querySelectorAll("br").forEach((node) => node.replaceWith("\n"));
  clone
    .querySelectorAll("p, li, h1, h2, h3, h4, h5, h6, tr, blockquote, div, hr")
    .forEach((node) => node.append("\n"));
  clone.querySelectorAll("td, th").forEach((node) => node.append(" "));
  return toSpeechText(clone.textContent ?? "");
}
