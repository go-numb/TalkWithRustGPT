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
import "./App.css";
import { Flex, Space, Row, Col, Button, Select, Image, Form, Input, Typography, message } from "antd";
const { TextArea } = Input;

import { prompts_list } from "./components/prompts";

// Voice API
import "regenerator-runtime/runtime";
import SpeechRecognition, { useSpeechRecognition } from 'react-speech-recognition';

import hljs from 'highlight.js';
import 'highlight.js/styles/default.css';
import { DrugComponent } from "./components/drug";
import { ImageComponent, resizeImageAndConvertToBase64 } from "./components/image";
import { sliceText } from "./common/string";

type Fields = {
  b?: number;
  msg?: string;
};

interface ResponseImage {
  prompt: string;
  url: string;
}

export const App = () => {
  const [messageApi, contextHolder] = message.useMessage();
  const [form] = Form.useForm();
  const [resultImageUrl, setResultImageUrl] = useState<string | null>(null);
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [imageUrls, setImageUrls] = useState<string[]>([]);
  const [isUpload, setIsUpload] = useState<boolean>(false);

  const MAX_WIDTH = 512; // 例として512pxに設定
  const MAX_HEIGHT = 512; // 例として512pxに設定


  const StatusNotSupport = "❌ Browser doesn't support speech recognition."
  const StatusAvailable = "❌ Microphone function is off, access to microphone is required."

  const StatusNone = ""
  const StatusListen = "🎧 Listening..."
  const StatusStop = "🎧 Stoped listening."
  const StatusStart = "🎧 Start listening."
  const StatusThinking = "🤖 Thinking..."
  const StatusModelLow = "🤖 Switch to model Economical."
  const StatusModelHigh = "🤖 Switch to model Performance."
  const StatusAIChatGPT = "🤖 Switch to ChatGPT."
  const StatusAIClaude = "🤖 Switch to Claude."
  const StatusAIGemini = "🤖 Switch to Gemini."
  const StatusResetMessages = "📝 Done! reset message history."

  const {
    transcript,
    listening,
    resetTranscript,
    browserSupportsSpeechRecognition,
    isMicrophoneAvailable,
  } = useSpeechRecognition();

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
    const code = document.querySelectorAll("pre code");
    code.forEach((block) => {
      hljs.highlightBlock(block as HTMLElement);
    });
  }, [result]);

  if (!browserSupportsSpeechRecognition) {
    setStatus(StatusNotSupport);
    return <span>{StatusNotSupport}</span>;
  }

  if (!isMicrophoneAvailable) {
    setStatus(StatusAvailable);
    return <span>{StatusAvailable}</span>;
  }

  const speech = () => {
    if (!listening) {
      SpeechRecognition.startListening({
        language: 'ja',
        continuous: true
      });
      setStatus(StatusStart);
    } else {
      resetTranscript();
      SpeechRecognition.startListening({
        language: 'ja',
        continuous: false
      });
      SpeechRecognition.stopListening();
      setStatus(StatusStop);
    }
  }

  useEffect(() => { // 音声認識が開始されたら、入力フォームにフォーカス
    if (listening) {
      setStatus(StatusListen);

      let set_text = transcript;
      form.setFieldValue("msg", set_text);

      let [is_there, command] = is_command_enter(set_text);
      if (is_there) {
        console.debug("command enter");
        resetTranscript();


        let reqest = set_text.replace(command, "");
        to_request(reqest);
      }
    }
  }, [transcript]);

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
        reset_all_vers();
        setQuery(`<h2 class="line_wrap">${prompt}</h2>\n`);
        if (!listening) {
          setStatus(StatusNone);
        }
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
        reset_all_vers();
        setQuery(`<h2 class="line_wrap">historical messages RAW: ${isRaw}</h2>\n`);
        if (!listening) {
          setStatus(StatusNone);
        }
      });
  }

  const to_request = async (req: string) => {
    const request_message = req === "" ? form.getFieldValue("msg") : req;

    console.debug(request_message);

    if (request_message === "") {
      setResult("Please enter a msg.");
      return;
    }
    setStatus(StatusThinking);

    // コマンドの処理
    const command = request_message.trim();
    if (command === "/raw") {
      // マークダウン整形せず出力
      get_all_messages(true);
      return;
    } else if (command === "/all") {
      // マークダウン整形して出力
      get_all_messages(false);
      return;
    } else if (command.includes("/image")) {
      // remove /dell3
      const prompt = command.replace("/image", "");
      get_image_to_dell3(prompt);
      return;
    }

    let src = "";
    if (imageUrl && !isUpload) {
      src = imageUrl;
      setImageUrls((prev) => [...prev, imageUrl]);

      setIsUpload(true);
      setImageUrl(null);
    }

    const to_invoke = AI === 0 ? "claude_request" : AI === 1 ? "chatgpt_request" : "gemini_request";
    console.log(`invoke: ${to_invoke}`);


    invoke(to_invoke, { b: model, msg: request_message, src: src })
      .then((res: any) => { // Add type annotation to 'res'
        console.debug(res);

        setResult(`${res}`);
      })
      .catch((err: any) => {
        console.error(`gemini_request > ${err}`);

        setStatus(`error: ${err}`);
      })
      .finally(() => {
        reset_all_vers();
        setQuery(`Q: ${request_message}`);
        if (!listening) {
          setStatus(StatusNone);
        }

        // 
      });
  }

  const reset_messages = () => {
    memo();
    invoke("reset");
    setQuery("");
    setImageUrls([]);
    setStatus(StatusResetMessages);
  };

  // リセット及びクローズとともにメモを作成する
  const memo = () => {
    invoke("memo")
      .then((message: any) => {
        setResult(`${message}`);
      })
      .catch((err: any) => {
        console.error(`memo > ${err}`);
        setResult(err);
      });
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

  // Usefull functions
  const reset_all_vers = () => {
    console.debug("reset_all_vers");

    resetTranscript();
    setImageUrl(null);
    form.setFieldValue("msg", "");

    // 画面のスクロールを最上部に移動
    window.scrollTo(0, 0);
    // カーソルをtextareaに移動
    inputRef.current?.focus();
  }

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
    console.debug("onDrop from component: ", insertStr);
    // 厳格に、insertStrが既に含まれていない場合のみ追加
    const prevMsg = (form.getFieldValue("msg") as string) || "";
    if (!prevMsg.includes(insertStr)) {
      const newMsg = prevMsg ? prevMsg + '\n' + insertStr : insertStr;
      form.setFieldValue("msg", newMsg);
    }
  };

  return (
    <Flex gap="large" vertical>
      {contextHolder}
      <DrugComponent onFileDrop={onDrop} />
      {/* 上部固定 */}
      <Flex className="fixed-left-bottom" gap="large" justify="end" align="center" vertical={false} >
        <Image
          height={40}
          src="/delete.png"
          onClick={reset_messages}
          alt="reset message logo"
          title="reset messages & save to file"
          className="reset message"
          preview={false}
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
        <Image
          height={40}
          src="/vc.png"
          onClick={speech}
          alt="vc logo"
          title="start/end vc for message"
          className="vc"
          preview={false}
        />
      </Flex>

      <Typography style={{ lineHeight: "1.75" }}>
        <Flex wrap vertical={false} gap="large" justify="center">
          <Row>
            {/* 質問内容の表示 */}
            {query && (

              <Col span={24}>
                <div
                  className="line_wrap"
                  title={query}
                  dangerouslySetInnerHTML={{ __html: `<h3 class="line_wrap">${sliceText(query, 25)}<h3>` }}
                  onClick={() => {
                    navigator.clipboard.writeText(query);
                    messageApi.success("Copied to clipboard!");
                  }}
                />
              </Col>
            )}

            {/* 回答内容の表示 */}
            {result && (
              <Col span={24}>
                <div
                  className="code-container markdown-body"
                  dangerouslySetInnerHTML={{ __html: result }}
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
          defaultValue={prompts_list[0].label}
          style={{ width: "100%" }}
          options={prompts_list}
          onChange={(value) => {
            if (!value || value === "None") return;

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
      </Flex>

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
            placeholder="Enter a msg..."
            onPasteCapture={async (e) => {
              if (!e.clipboardData.files.length) return;
              e.preventDefault();
              const file = e.clipboardData.files[0];
              if (file) {
                const base64 = await resizeImageAndConvertToBase64(file, MAX_WIDTH, MAX_HEIGHT);
                setImageUrl(base64);
                setIsUpload(false);
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

        <Form.Item wrapperCol={{ offset: 21, span: 3 }}>
          <Button type="primary" htmlType="submit">
            SEND
          </Button>
        </Form.Item>
      </Form>

      <Space className="footer-fixed">
        {status}
      </Space>

    </Flex>
  );
};