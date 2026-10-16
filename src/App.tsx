/**
 * メインアプリケーションコンポーネント。
 * 
 * - 音声認識、画像アップロード、AIモデル切り替え、プロンプト選択などの機能を提供します。
 * - Tauriバックエンドとの通信を行い、AIサービス（Claude, ChatGPT, Gemini）へのリクエストを管理します。
 * - 入力フォーム、音声認識、画像リサイズ・表示、メッセージ履歴管理などのUIを含みます。
 * 
 * @component
 * @returns {JSX.Element} アプリケーションのルート要素
 */

/**
 * 画像を表示するコンポーネント。
 * 
 * @param {Object} props
 * @param {string[]} props.images - 表示する画像のBase64またはURL配列
 * @param {number} props.size - 画像の表示サイズ（px）
 * @returns {JSX.Element[]} 画像要素の配列
 */

/**
 * 画像ファイルをリサイズし、Base64データURLに変換する関数。
 * 
 * @param {File} file - 入力画像ファイル
 * @param {number} maxWidth - 最大幅（px）
 * @param {number} maxHeight - 最大高さ（px）
 * @returns {Promise<string>} リサイズ後のBase64データURL
 */
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import "./App.css";
import { Flex, Space, Row, Col, Button, Select, Image, Form, Input, Tabs, Typography, message } from "antd";
import { LoadingOutlined, MutedOutlined, SoundOutlined } from "@ant-design/icons";
const { TextArea } = Input;

import { prompts_list } from "./components/prompts";

// Voice API
import "regenerator-runtime/runtime";
import SpeechRecognition, { useSpeechRecognition } from 'react-speech-recognition';

import hljs from 'highlight.js';
import 'highlight.js/styles/default.css';
import 'katex/dist/katex.min.css';
import { DrugComponent } from "./components/drug";
import { SettingsComponent } from "./components/settings";
import { ImageComponent, resizeImageAndConvertToBase64 } from "./components/image";
import { sliceText } from "./common/string";
import { renderMath } from "./common/math";
import { RequestQueue } from "./common/requestQueue";
import {
  SpeechSession, TTS_VOICES, TtsVoice, createWebAudioPlayer, extractSpeechText, isTtsVoice, splitForSpeech,
  toArrayBuffer,
} from "./common/speech";
import {
  BUILTIN_TTS_DEFAULTS, TtsSettings, loadSavedTtsSettings, mergeTtsSettings, storeSavedTtsSettings,
} from "./common/settings";

type Fields = {
  b?: number;
  msg?: string;
};

interface ResponseImage {
  prompt: string;
  url: string;
}

interface ChatRequest {
  readonly msg: string;
  readonly src: string;
  readonly ai: number;
  readonly model: number;
}

const AI_NAMES = ["Claude", "ChatGPT", "Gemini"];
const TTS_PREVIEW_TEXT = "こんにちは。選択した声のサンプルです。This is a sample of the selected voice.";

export const App = () => {
  const [messageApi, contextHolder] = message.useMessage();
  const [form] = Form.useForm();
  const [resultImageUrl, setResultImageUrl] = useState<string | null>(null);
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [imageUrls, setImageUrls] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isResetting, setIsResetting] = useState(false);
  // React の再描画前の連打や、音声送信とリセットの競合も防ぐ。
  const isBusyRef = useRef(false);
  const [requestQueue] = useState(() => new RequestQueue<ChatRequest>());
  const [queuedRequest, setQueuedRequest] = useState<ChatRequest | null>(null);
  const [failedRequest, setFailedRequest] = useState<ChatRequest | null>(null);
  const [isProcessingImage, setIsProcessingImage] = useState(false);
  const imageProcessingRef = useRef(false);

  useEffect(() => () => { requestQueue.cancel(); }, [requestQueue]);

  // AI 回答の読み上げ (Gemini TTS)
  const resultRef = useRef<HTMLDivElement>(null);
  const [speechSession] = useState(() => new SpeechSession());
  const [audioPlayer] = useState(() => createWebAudioPlayer());
  const [speechState, setSpeechState] = useState<"idle" | "loading" | "playing">("idle");
  const [ttsVoice, setTtsVoice] = useState<TtsVoice>(() => {
    const saved = localStorage.getItem("ttsVoice");
    return isTtsVoice(saved) ? saved : "female";
  });

  // 読み上げの対象。answer = 表示中の回答、female / male = 設定タブでの試聴
  const [speechTarget, setSpeechTarget] = useState<"answer" | TtsVoice | null>(null);
  // 設定タブで変更した項目 (savedTts) を、環境変数を反映した既定値 (ttsDefaults) に重ねて使う
  const [ttsDefaults, setTtsDefaults] = useState<TtsSettings>(BUILTIN_TTS_DEFAULTS);
  const [savedTts, setSavedTts] = useState<Partial<TtsSettings>>(loadSavedTtsSettings);
  const ttsSettings = mergeTtsSettings(ttsDefaults, savedTts);
  const voiceNameOf = (voice: TtsVoice) => voice === "female" ? ttsSettings.femaleVoice : ttsSettings.maleVoice;
  const [activeTab, setActiveTab] = useState("chat");

  useEffect(() => () => { speechSession.stop(); }, [speechSession]);

  useEffect(() => {
    invoke<TtsSettings>("tts_defaults")
      .then((defaults) => setTtsDefaults(mergeTtsSettings(BUILTIN_TTS_DEFAULTS, defaults)))
      .catch((err) => console.error(`tts_defaults > ${err}`));
  }, []);

  const IMAGE_SIZE_OPTIONS = [512, 1024, 2048];
  const DEFAULT_IMAGE_SIZE = 1024;
  const [imageMaxSize, setImageMaxSize] = useState<number>(() => {
    const saved = localStorage.getItem("imageMaxSize");
    return saved ? Number(saved) : DEFAULT_IMAGE_SIZE;
  });


  const StatusNotSupport = "この環境では音声入力に対応していません。テキスト入力は利用できます。"
  const StatusMicrophoneUnavailable = "マイクが利用できません。テキスト入力は利用できます。音声入力を使うには、マイクの設定・権限を確認してアプリを再起動してください。"

  const StatusNone = ""
  const StatusListen = "🎧 Listening..."
  const StatusStop = "🎧 Stopped listening."
  const StatusThinking = "🤖 Thinking..."
  const StatusModelLow = "🤖 Switch to model Economical."
  const StatusModelHigh = "🤖 Switch to model Performance."
  const StatusAIChatGPT = "🤖 Switch to ChatGPT."
  const StatusAIClaude = "🤖 Switch to Claude."
  const StatusAIGemini = "🤖 Switch to Gemini."
  const StatusResetMessages = "📝 Done! reset message history."
  const StatusSpeechLoading = "🔊 Generating voice..."
  const StatusSpeaking = "🔊 Speaking..."

  const {
    transcript,
    listening,
    resetTranscript,
    browserSupportsSpeechRecognition,
    isMicrophoneAvailable,
  } = useSpeechRecognition();

  const speechUnavailableMessage = !browserSupportsSpeechRecognition
    ? StatusNotSupport
    : !isMicrophoneAvailable ? StatusMicrophoneUnavailable : "";
  const voiceIsActive = listening && !speechUnavailableMessage;

  const [query, setQuery] = useState("");
  const [result, setResult] = useState("");
  const [model, setModel] = useState<number>(0);
  // set gemini
  const [AI, setAI] = useState<number>(2);
  const [status, setStatus] = useState(StatusModelHigh);

  const inputRef = useRef<HTMLInputElement>(null);

  // 起動時に、環境変数: CHATGPTTOKEN、ANTHROPIC_API_KEYどちらもなければ、setResultにエラーメッセージを表示する
  const init_check = async () => {
    const isEnvAvailable = await invoke("is_there_env");
    if (isEnvAvailable !== true) {
      setResult(`[ALERT]ご利用できません: 各AIサービスを利用するための環境変数: CHATGPTTOKENまたは ANTHROPIC_API_KEYを設定してください。`);
    }
  };


  useEffect(() => {
    // ReferenceError: process is not defined
    init_check();
  }, []);

  // useEffect 変数監視セクション
  useEffect(() => { // Resultが更新され、Queryが刷新されたら、入力フォームにフォーカス
    // 入力フォームにフォーカス
    // const textField = document.getElementById("input-msg") as HTMLInputElement;
    // textField?.focus();
    window.scrollTo(0, 0);
  }, [query]);

  // resultの内容をhighlight.jsでハイライトする
  useEffect(() => {
    // 表示中の回答が変わったら、前の回答の読み上げは止める
    stopSpeaking();
    const code = document.querySelectorAll("pre code");
    code.forEach((block) => {
      hljs.highlightBlock(block as HTMLElement);
    });
  }, [result]);

  useEffect(() => {
    if (voiceIsActive) {
      setStatus(StatusListen);
    } else {
      setStatus((previous) => previous === StatusListen ? StatusNone : previous);
    }
  }, [voiceIsActive]);

  const speech = async () => {
    if (speechUnavailableMessage || requestQueue.pending) return;

    try {
      if (voiceIsActive) {
        await SpeechRecognition.stopListening();
        resetTranscript();
        setStatus(StatusStop);
      } else {
        // 読み上げ音声をマイクが拾って入力・送信しないよう、読み上げを止める
        stopSpeaking();
        await SpeechRecognition.startListening({
          language: 'ja',
          continuous: true
        });
      }
    } catch (err) {
      setStatus(`音声入力を切り替えられませんでした: ${String(err)}`);
    }
  }

  useEffect(() => { // 音声認識が開始されたら、入力フォームにフォーカス
    if (voiceIsActive && !requestQueue.pending) {
      let set_text = transcript;
      form.setFieldValue("msg", set_text);

      let [is_there, command] = is_command_enter(set_text);
      if (is_there) {
        console.debug("command enter");
        let reqest = set_text.replace(command, "");
        to_request(reqest);
      }
    }
  }, [transcript]);

  const stopSpeaking = () => {
    speechSession.stop();
    setSpeechState("idle");
    setSpeechTarget(null);
    setStatus((previous) =>
      previous === StatusSpeechLoading || previous === StatusSpeaking ? StatusNone : previous);
  };

  const speak = async (text: string, target: "answer" | TtsVoice, voice: string) => {
    // 自動再生制限を避けるため、await より前のクリック処理内で音声出力を有効にする
    audioPlayer.unlock();
    const model = ttsSettings.model;
    setSpeechTarget(target);
    setSpeechState("loading");
    setStatus(StatusSpeechLoading);
    if (voiceIsActive) {
      // 読み上げ音声をマイクが拾って入力・送信しないよう、音声入力を止めてから読み上げる
      try {
        await SpeechRecognition.stopListening();
        resetTranscript();
      } catch (err) {
        setSpeechState("idle");
        setSpeechTarget(null);
        setStatus(`音声入力を停止できないため、読み上げを中止しました: ${String(err)}`);
        return;
      }
    }

    try {
      const completed = await speechSession.speak(
        splitForSpeech(text),
        (chunk) => invoke<ArrayBuffer | number[]>("gemini_tts", { text: chunk, voice, model }).then(toArrayBuffer),
        audioPlayer.play,
        () => {
          setSpeechState("playing");
          setStatus(StatusSpeaking);
        },
      );
      if (completed) {
        setStatus((previous) =>
          previous === StatusSpeechLoading || previous === StatusSpeaking ? StatusNone : previous);
      }
    } catch (err) {
      setStatus(`読み上げに失敗しました: ${String(err)}`);
    } finally {
      // 停止直後に次の読み上げが始まっている場合は、その状態を上書きしない
      if (!speechSession.active) {
        setSpeechState("idle");
        setSpeechTarget(null);
      }
    }
  };

  const speak_result = () => {
    if (speechSession.active && speechTarget === "answer") {
      stopSpeaking();
      return;
    }
    const text = resultRef.current ? extractSpeechText(resultRef.current) : "";
    if (!text) {
      messageApi.info("読み上げる回答がありません。");
      return;
    }
    // 試聴中なら speak() が試聴を止めて回答の読み上げに切り替える
    void speak(text, "answer", voiceNameOf(ttsVoice));
  };

  const preview_voice = (voice: TtsVoice) => {
    if (speechSession.active && speechTarget === voice) {
      stopSpeaking();
      return;
    }
    void speak(TTS_PREVIEW_TEXT, voice, voiceNameOf(voice));
  };

  const update_tts_settings = (patch: Partial<TtsSettings>) => {
    const next = { ...savedTts, ...patch };
    setSavedTts(next);
    storeSavedTtsSettings(next);
  };

  const reset_tts_settings = () => {
    setSavedTts({});
    storeSavedTtsSettings({});
  };

  const get_image_to_dell3 = (prompt: string) => {
    invoke("chatgpt_request_to_dell3", { size: 1, msg: prompt })
      .then((res: any) => {
        let image = JSON.parse(res as string) as ResponseImage;
        prompt = prompt + " to prompt, " + image.prompt;
        console.debug(image);
        setResult(`${prompt}`);
        setResultImageUrl(image.url);
      })
      .catch((err: any) => {
        console.error(`chatgpt_request_to_dell3 > ${err}`);

        setStatus(`error: ${err}`);
      })
      .finally(() => {
        isBusyRef.current = false;
        setIsLoading(false);
        setQuery(`[image] ${prompt}`);
        setStatus((previous) => previous === StatusThinking ? StatusNone : previous);
      });
  }

  const get_all_messages = (isRaw: boolean) => {
    invoke("all_messages", { is_raw: isRaw })
      .then((res: any) => {
        console.debug(res);

        setResult(`${res}`);
      })
      .catch((err: any) => {
        console.error(`get_all_messages > ${err}`);

        setStatus(`error: ${err}`);
      })
      .finally(() => {
        isBusyRef.current = false;
        setIsLoading(false);
        setQuery(`[history] RAW: ${isRaw}`);
        setStatus((previous) => previous === StatusThinking ? StatusNone : previous);
      });
  }

  const send_chat = async (initial: ChatRequest) => {
    let currentRequest = initial;
    setFailedRequest(null);
    try {
      await requestQueue.run(initial, async (request) => {
        currentRequest = request;
        // A reserved draft is consumed only when its request actually starts.
        setQueuedRequest(null);
        clearSubmittedDraft(true);
        setStatus(StatusThinking);
        if (request.src) setImageUrls((previous) => [...previous, request.src]);

        const command = request.ai === 0 ? "claude_request"
          : request.ai === 1 ? "chatgpt_request" : "gemini_request";
        const response = await invoke<string>(command, {
          b: request.model, msg: request.msg, src: request.src,
        });
        setResult(response);
        setResultImageUrl(null);
        setQuery(`Q: ${request.msg}`);
      });
      setStatus((previous) => previous === StatusThinking ? StatusNone : previous);
    } catch (err) {
      // The backend may already contain the failed user message; never retry automatically.
      setFailedRequest(currentRequest);
      setStatus(`送信に失敗しました。送信予約は解除され、下書きは保持されています: ${String(err)}`);
    } finally {
      setQueuedRequest(null);
      isBusyRef.current = false;
      setIsLoading(false);
    }
  };

  const to_request = async (req: string) => {
    if (requestQueue.pending || imageProcessingRef.current) return;
    if (isBusyRef.current && !requestQueue.isRunning) return;

    const request_message = req === "" ? (form.getFieldValue("msg") || "") : req;
    const command = request_message.trim();
    if (command === "") {
      messageApi.info("メッセージを入力してください。");
      return;
    }

    const isImageCommand = /^\/image(?:\s|$)/.test(command);
    const isLocalCommand = command === "/raw" || command === "/all" || isImageCommand;
    const request: ChatRequest = { msg: request_message, src: imageUrl || "", ai: AI, model };

    if (requestQueue.isRunning) {
      if (isLocalCommand) {
        messageApi.info("コマンドは予約できません。応答が完了してから実行してください。");
        return;
      }
      if (requestQueue.reserve(request)) {
        form.setFieldValue("msg", request.msg);
        setQueuedRequest(request);
        if (voiceIsActive) {
          SpeechRecognition.stopListening().then(resetTranscript).catch((err) => {
            setStatus(`音声入力を停止できませんでした: ${String(err)}`);
          });
        }
      }
      return;
    }

    isBusyRef.current = true;
    setIsLoading(true);
    setStatus(StatusThinking);

    if (isLocalCommand) {
      // These commands do not consume an attached image.
      clearSubmittedDraft(false);
      if (command === "/raw" || command === "/all") {
        get_all_messages(command === "/raw");
      } else {
        get_image_to_dell3(command.slice("/image".length).trim());
      }
      return;
    }

    await send_chat(request);
  };

  const cancel_reservation = () => {
    requestQueue.cancel();
    setQueuedRequest(null);
    inputRef.current?.focus();
  };

  const reset_messages = async () => {
    if (isBusyRef.current) return;

    isBusyRef.current = true;
    setIsResetting(true);
    setStatus("📝 Saving conversation...");
    try {
      await invoke("memo_and_reset");
      setQuery("");
      setResult("");
      setImageUrls([]);
      setResultImageUrl(null);
      setFailedRequest(null);
      setStatus(StatusResetMessages);
    } catch (err) {
      setStatus(`保存に失敗しました。会話履歴は保持されています: ${String(err)}`);
    } finally {
      isBusyRef.current = false;
      setIsResetting(false);
    }
  };

  const switch_model = () => {
    if (model != 0) {
      setModel(0);
      setStatus(StatusModelLow);
    } else {
      setModel(1);
      setStatus(StatusModelHigh);
    }
  }

  const switch_ai = () => {
    setAI((prev) => {
      prev++;
      if (prev > 2) {
        prev = 0;
      }
      switch (prev) {
        case 0:
          setStatus(StatusAIClaude);
          break;
        case 1:
          setStatus(StatusAIChatGPT);
          break;
        default:
          setStatus(StatusAIGemini);
      }
      return prev;
    });
  }

  const clearSubmittedDraft = (includeImage: boolean) => {
    resetTranscript();
    if (includeImage) setImageUrl(null);
    form.setFieldValue("msg", "");
    inputRef.current?.focus();
  };

  const is_command_enter = (str: string): [Boolean, string] => {
    let command_str = str;
    if (command_str.endsWith("エンター。")) {
      return [true, "エンター"];
    } else if (command_str.endsWith("送信。")) {
      return [true, "送信"];
    } else if (command_str.endsWith("教えて。")) {
      return [true, ""];
    }

    return [false, ""];
  }

  const request_system = (num: number) => {
    return () => {
      invoke("request_system", { num: num })
        .then((res: any) => {
          setStatus(`${res}`);
        })
        .catch((err: any) => {
          console.error(`request_system > ${err}`);

          setStatus(`error: ${err}`);
        });
    }
  }

  const change_icon = (): string => {
    switch (AI) {
      case 0:
        return "/claude-ai.png";
      case 1:
        return "/chatgpt-ai.png";
      default:
        return "/gemini-ai.png";
    };
  }

  const onDrop = (insertStr: string): void => {
    if (requestQueue.pending) return;
    console.debug("onDrop from component: ", insertStr);
    // 厳格に、insertStrが既に含まれていない場合のみ追加
    const prevMsg = (form.getFieldValue("msg") as string) || "";
    if (!prevMsg.includes(insertStr)) {
      const newMsg = prevMsg ? prevMsg + '\n' + insertStr : insertStr;
      form.setFieldValue("msg", newMsg);
    }
  };

  const answerSpeech = speechTarget === "answer" ? speechState : "idle";
  const preview = speechTarget && speechTarget !== "answer" && speechState !== "idle"
    ? { voice: speechTarget, state: speechState }
    : null;

  const chatTab = (
    <Flex gap="large" vertical>
      {/* 上部固定 */}
      <Flex className="fixed-left-bottom" gap="large" justify="end" align="center" vertical={false} >
        <Button
          type="text"
          onClick={reset_messages}
          disabled={isLoading || isResetting}
          aria-label="Save and reset conversation"
          title="reset messages & save to file"
          className="reset message"
          style={{ height: 40, padding: 0 }}
          icon={<img height={40} src="/delete.png" alt="" />}
        />
        <Image
          height={40}
          src={model === 0 ? "/switch-model-high.png" : "/switch-model-low.png"}
          onClick={switch_model}
          alt="switch model logo"
          title="switch set model"
          className="switch model"
          preview={false}
        />
        <Image
          height={40}
          src={change_icon()}
          onClick={switch_ai}
          alt="switch ai logo"
          title="switch set ai"
          className="switch ai"
          preview={false}
        />
        <Button
          type="text"
          onClick={speech}
          disabled={Boolean(speechUnavailableMessage) || Boolean(queuedRequest)}
          aria-label={voiceIsActive ? "Stop voice input" : "Start voice input"}
          aria-pressed={voiceIsActive}
          aria-describedby={speechUnavailableMessage ? "voice-unavailable" : undefined}
          title={speechUnavailableMessage || "start/end vc for message"}
          className="vc"
          style={{ height: 40, padding: 0 }}
          icon={<img height={40} src="/vc.png" alt="" />}
        />
      </Flex>

      <Typography style={{ lineHeight: "1.75" }}>
        <Flex wrap vertical={false} gap="large" justify="center">
          <Row>
            {/* 質問内容の表示 */}
            {query && (

              <Col span={24}>
                <h3
                  className="line_wrap"
                  title={query}
                  onClick={() => {
                    navigator.clipboard.writeText(query);
                    messageApi.success("Copied to clipboard!");
                  }}
                  style={{ cursor: "pointer" }}
                >
                  {sliceText(query, 25)}
                </h3>
              </Col>
            )}

            {/* 回答内容の表示 */}
            {result && (
              <Col span={24}>
                <div
                  ref={resultRef}
                  className="code-container markdown-body"
                  onClick={(e) => {
                    const anchor = (e.target as HTMLElement).closest("a");
                    if (anchor && anchor.href) {
                      e.preventDefault();
                      openUrl(anchor.href).catch(() => {
                        /* opener が失敗しても WebView は遷移させない */
                      });
                    }
                  }}
                  dangerouslySetInnerHTML={{ __html: renderMath(result) }}
                />
              </Col>

            )}
          </Row>
          {/* 画像の表示 */}
          {resultImageUrl && (
            <ImageComponent images={[resultImageUrl]} size={1024} />
          )}
        </Flex>
      </Typography>

      <Flex gap="large" justify="space-between">
        {/* 用途による文面を自動挿入 */}
        <Select
          disabled={Boolean(queuedRequest)}
          defaultValue={prompts_list[0].label}
          style={{ width: "100%" }}
          options={prompts_list}
          onChange={(value) => {
            if (requestQueue.pending || !value || value === "None") return;

            // Rust側にリクエストを送信
            // Rust処理（リクエスト・出力）と、GUI文字挿入がある。
            const numValue = Number(value);
            if (!isNaN(numValue)) {
              request_system(numValue)();
              return;
            }

            form.setFieldValue("msg", value);
          }}
        />
        {/* 表示中の回答を読み上げる (声とモデルは設定タブで選ぶ) */}
        <Button
          onClick={speak_result}
          disabled={!result}
          aria-label={answerSpeech === "idle" ? "Read the answer aloud" : "Stop reading aloud"}
          aria-pressed={answerSpeech !== "idle"}
          title={answerSpeech === "idle"
            ? `回答を読み上げる（${TTS_VOICES.find((v) => v.value === ttsVoice)?.label}: ${voiceNameOf(ttsVoice)}）`
            : "読み上げを停止"}
          className="tts"
          style={{ flexShrink: 0 }}
          icon={answerSpeech === "loading" ? <LoadingOutlined />
            : answerSpeech === "playing" ? <MutedOutlined /> : <SoundOutlined />}
        />
      </Flex>

      {speechUnavailableMessage && (
        <Typography.Text id="voice-unavailable" type="secondary" role="status">
          {speechUnavailableMessage}
        </Typography.Text>
      )}

      <Form
        name="basic"
        form={form}
        wrapperCol={{ span: 24 }}
        className="form"
        onFinish={() => to_request("")}
      >
        <Form.Item<Fields> name="msg" wrapperCol={{ span: 24 }}>
          <TextArea
            ref={inputRef}
            rows={4}
            placeholder={isLoading ? "次の質問を入力できます…" : "Enter a msg..."}
            readOnly={Boolean(queuedRequest)}
            onPasteCapture={async (e) => {
              if (requestQueue.pending) {
                e.preventDefault();
                return;
              }
              if (!e.clipboardData.files.length) return;
              e.preventDefault();
              if (imageProcessingRef.current) return;
              const file = e.clipboardData.files[0];
              if (!file.type.startsWith("image/")) {
                messageApi.error("画像ファイルを貼り付けてください。");
                return;
              }
              imageProcessingRef.current = true;
              setIsProcessingImage(true);
              try {
                const base64 = await resizeImageAndConvertToBase64(file, imageMaxSize, imageMaxSize);
                setImageUrl(base64);
              } catch (err) {
                messageApi.error(`画像を読み込めませんでした: ${String(err)}`);
              } finally {
                imageProcessingRef.current = false;
                setIsProcessingImage(false);
              }
            }}
          />
        </Form.Item>

        <Flex gap="large">
          <Row>
            <Col>
              <ImageComponent images={imageUrl ? [imageUrl] : []} size={200} />
              <Flex wrap>
                <ImageComponent images={imageUrls ?? []} size={58} />
              </Flex>
            </Col>
          </Row>
        </Flex>

        {failedRequest && (
          <details>
            <summary>送信に失敗した質問</summary>
            <Typography.Paragraph copyable={{ text: failedRequest.msg }} style={{ whiteSpace: "pre-wrap" }}>
              {failedRequest.msg}
            </Typography.Paragraph>
            {failedRequest.src && <ImageComponent images={[failedRequest.src]} size={200} />}
          </details>
        )}
        {queuedRequest && (
          <Flex gap="small" align="center" justify="space-between" wrap>
            <Typography.Text role="status">
              送信予約: {AI_NAMES[queuedRequest.ai]} / {queuedRequest.model === 1 ? "高性能" : "経済"}
            </Typography.Text>
            <Button onClick={cancel_reservation}>予約を取り消して編集</Button>
          </Flex>
        )}
        {isProcessingImage && <Typography.Text role="status">画像を準備中…</Typography.Text>}
        {isLoading && requestQueue.isRunning && !queuedRequest && (
          <Typography.Text type="secondary" role="status">
            回答を待っています。次の質問を1件予約できます。
          </Typography.Text>
        )}
        <Form.Item style={{ textAlign: "right" }}>
          <Button
            type="primary"
            htmlType="submit"
            loading={isLoading && !requestQueue.isRunning}
            disabled={isResetting || Boolean(queuedRequest) || isProcessingImage || (isLoading && !requestQueue.isRunning)}
          >
            {queuedRequest ? "予約済み" : requestQueue.isRunning ? "次に送信" : "SEND"}
          </Button>
        </Form.Item>
      </Form>

    </Flex>
  );

  const settingsTab = (
    <SettingsComponent
      tts={ttsSettings}
      ttsVoice={ttsVoice}
      isTtsCustomized={Object.keys(savedTts).length > 0}
      imageMaxSize={imageMaxSize}
      imageSizeOptions={IMAGE_SIZE_OPTIONS}
      preview={preview}
      onTtsChange={update_tts_settings}
      onTtsVoiceChange={(voice) => {
        setTtsVoice(voice);
        localStorage.setItem("ttsVoice", voice);
      }}
      onTtsReset={reset_tts_settings}
      onPreview={preview_voice}
      onImageMaxSizeChange={(value) => {
        setImageMaxSize(value);
        localStorage.setItem("imageMaxSize", String(value));
      }}
    />
  );

  return (
    <Flex gap="large" vertical>
      {contextHolder}
      <DrugComponent onFileDrop={onDrop} />
      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
        items={[
          { key: "chat", label: "チャット", children: chatTab },
          { key: "settings", label: "設定", children: settingsTab },
        ]}
      />

      <Space className="footer-fixed">
        {status}
      </Space>

    </Flex>
  );
};
