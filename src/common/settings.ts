// 設定タブで管理する読み上げ設定。
// 優先順位: 設定タブで変更した値 (localStorage) > 環境変数 (Rust の tts_defaults) > 既定値。

export interface TtsSettings {
  model: string;
  femaleVoice: string;
  maleVoice: string;
}

export type TtsOption = { value: string; label: string };

export const BUILTIN_TTS_DEFAULTS: TtsSettings = {
  model: "gemini-3.8-flash-tts",
  femaleVoice: "Erinome",
  maleVoice: "Charon",
};

export const TTS_MODELS: readonly TtsOption[] = [
  { value: "gemini-3.8-flash-tts", label: "Gemini 3.8 Flash TTS（高品質）" },
  { value: "gemini-3.8-flash-lite-tts", label: "Gemini 3.8 Flash-Lite TTS（低遅延・低コスト）" },
];

// 多言語のプリセット音声。性別は Voice Library (GET /v1beta/voices) の gender による。
export const FEMALE_VOICES: readonly TtsOption[] = [
  { value: "Erinome", label: "Erinome（Clear）" },
  { value: "Kore", label: "Kore（Firm）" },
  { value: "Aoede", label: "Aoede（Breezy）" },
  { value: "Achernar", label: "Achernar（Soft）" },
  { value: "Autonoe", label: "Autonoe（Bright）" },
  { value: "Callirrhoe", label: "Callirrhoe（Easy-going）" },
  { value: "Despina", label: "Despina（Smooth）" },
  { value: "Gacrux", label: "Gacrux（Mature）" },
  { value: "Laomedeia", label: "Laomedeia（Upbeat）" },
  { value: "Leda", label: "Leda（Youthful）" },
  { value: "Sulafat", label: "Sulafat（Warm）" },
  { value: "Vindemiatrix", label: "Vindemiatrix（Gentle）" },
  { value: "Zephyr", label: "Zephyr（Bright）" },
];

export const MALE_VOICES: readonly TtsOption[] = [
  { value: "Charon", label: "Charon（Informative）" },
  { value: "Iapetus", label: "Iapetus（Clear）" },
  { value: "Achird", label: "Achird（Friendly）" },
  { value: "Algenib", label: "Algenib（Gravelly）" },
  { value: "Algieba", label: "Algieba（Smooth）" },
  { value: "Alnilam", label: "Alnilam（Firm）" },
  { value: "Enceladus", label: "Enceladus（Breathy）" },
  { value: "Fenrir", label: "Fenrir（Excitable）" },
  { value: "Orus", label: "Orus（Firm）" },
  { value: "Puck", label: "Puck（Upbeat）" },
  { value: "Rasalgethi", label: "Rasalgethi（Informative）" },
  { value: "Sadachbia", label: "Sadachbia（Lively）" },
  { value: "Sadaltager", label: "Sadaltager（Knowledgeable）" },
  { value: "Schedar", label: "Schedar（Even）" },
  { value: "Umbriel", label: "Umbriel（Easy-going）" },
  { value: "Zubenelgenubi", label: "Zubenelgenubi（Casual）" },
];

// Rust 側 gemini_tts の検証と同じ規則。
const ID = /^[A-Za-z0-9._-]{1,64}$/;
export const isTtsId = (value: unknown): value is string => typeof value === "string" && ID.test(value);

const KEYS: readonly (keyof TtsSettings)[] = ["model", "femaleVoice", "maleVoice"];

/** 有効な値だけを base に重ねる。不正な値や未設定の項目は base のまま。 */
export function mergeTtsSettings(base: TtsSettings, override: unknown): TtsSettings {
  const merged = { ...base };
  if (typeof override !== "object" || override === null) return merged;
  for (const key of KEYS) {
    const value = (override as Record<string, unknown>)[key];
    if (isTtsId(value)) merged[key] = value;
  }
  return merged;
}

/** 設定タブで変更した項目だけを返す。壊れた JSON や不正な値は無視する。 */
export function parseSavedTtsSettings(json: string | null): Partial<TtsSettings> {
  let parsed: unknown;
  try {
    parsed = json ? JSON.parse(json) : null;
  } catch {
    return {};
  }
  if (typeof parsed !== "object" || parsed === null) return {};
  const saved: Partial<TtsSettings> = {};
  for (const key of KEYS) {
    const value = (parsed as Record<string, unknown>)[key];
    if (isTtsId(value)) saved[key] = value;
  }
  return saved;
}

/**
 * 一覧にない音声やモデル (環境変数で指定した既定値や現在の値) も選択肢に表示する。
 * 既定値を渡しておくと、別の値を選んだ後でも選び直せる。
 */
export function withCurrentOption(options: readonly TtsOption[], ...values: string[]): TtsOption[] {
  const extra = [...new Set(values)]
    .filter((value) => !options.some((option) => option.value === value))
    .map((value) => ({ value, label: value }));
  return [...extra, ...options];
}

const STORAGE_KEY = "ttsSettings";

export function loadSavedTtsSettings(): Partial<TtsSettings> {
  try {
    return parseSavedTtsSettings(localStorage.getItem(STORAGE_KEY));
  } catch {
    return {};
  }
}

export function storeSavedTtsSettings(saved: Partial<TtsSettings>): void {
  try {
    if (Object.keys(saved).length === 0) localStorage.removeItem(STORAGE_KEY);
    else localStorage.setItem(STORAGE_KEY, JSON.stringify(saved));
  } catch {
    // 保存できない環境でも、この起動中の設定は使える。
  }
}
