import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Flex, Space, Row, Col, Button, Select, Image, Form, Input } from "antd";
const { TextArea } = Input;

import { prompts_list } from "./components/prompts";

// Voice API
import "regenerator-runtime/runtime";
import SpeechRecognition, { useSpeechRecognition } from 'react-speech-recognition';

import hljs from 'highlight.js';
import 'highlight.js/styles/default.css';
import { DrugComponent } from "./components/drug";

type Fields = {
  b?: number;
  msg?: string;
};

interface ResponseImage {
  prompt: string;
  url: string;
}

function App() {
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
  const [msg, setMsg] = useState("");

  // // file upload
  // const [fileList, setFileList] = useState<UploadFile<any>[]>([]);

  const [query, setQuery] = useState("");
  const [result, setResult] = useState("");
  const [model, setModel] = useState<number>(1);
  const [AI, setAI] = useState<number>(0);
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
      setMsg(transcript);
      setStatus(StatusListen);

      form.setFieldValue("msg", transcript);

      console.debug(`listening: ${listening}, ${transcript}, msg: ${msg}`);

      let [is_there, command] = is_command_enter(transcript);
      if (is_there) {
        console.debug("command enter");
        resetTranscript();
        let reqest = msg.replace(command, "");
        setMsg(reqest);
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

  const get_all_messages = () => {
    invoke("all_messages")
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
        setQuery(`<h2 class="line_wrap">historical messages: </h2>\n`);
        if (!listening) {
          setStatus(StatusNone);
        }
      });
  }

  const to_request = async (req: string) => {
    let _msg = msg;
    if (req != "") {
      console.debug("req: ", req);

      _msg = req;
    }
    console.debug(_msg);

    if (_msg === "") {
      setResult("Please enter a msg.");
      return;
    }
    setStatus(StatusThinking);

    // コマンドの処理
    if (_msg === "/all") {
      get_all_messages();
      return;
    } else if (_msg.includes("/image")) {
      // remove /dell3
      const prompt = _msg.replace("/image", "");
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


    invoke(to_invoke, { b: model, msg: _msg, src: src })
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
        setQuery(`<h2 class="line_wrap">Q: ${_msg}</h2>\n`);
        if (!listening) {
          setStatus(StatusNone);
        }

        // 
      });
  }

  const reset_messages = () => {
    memo();
    invoke("reset");
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
    setMsg("");
    setImageUrl(null);
    form.setFieldValue("msg", "");

    // 画面のスクロールを最上部に移動
    window.scrollTo(0, 0);
    // カーソルをtextareaに移動
    inputRef.current?.focus();

    console.debug(msg);

  }

  const is_command_enter = (str: string): [Boolean, string] => {
    let _msg = str;
    if (_msg.endsWith("エンター。")) {
      return [true, "エンター"];
    } else if (_msg.endsWith("送信。")) {
      return [true, "送信"];
    } else if (_msg.endsWith("教えて。")) {
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
      <DrugComponent onFileDrop={onDrop} />
      <Flex gap={'large'} justify="space-between" vertical={false}>
        <Image preview={false} style={{ maxWidth: '128px' }} onClick={reset_messages} src="/delete.png" className="logo reset message" alt="reset message logo" title="reset messages & save to file" />
        <Image preview={false} style={{ maxWidth: '128px' }} onClick={switch_model} src={model === 0 ? "/switch-model-high.png" : "/switch-model-low.png"} className="logo switch model" alt="switch model logo" title="switch set model" />
        <Image preview={false} style={{ maxWidth: '128px' }} onClick={switch_ai} src={change_icon()} className="logo switch ai" alt="switch ai logo" title="switch set ai" />
        <Image preview={false} style={{ maxWidth: '128px' }} onClick={speech} src="/vc.png" className="logo vc" alt="vc logo" title="start/end vc for message" />
      </Flex>

      <Flex wrap vertical={false} gap={'large'} justify="center">
        <div className="line_wrap" dangerouslySetInnerHTML={{ __html: query }} />

        <div className="code-container markdown-body" dangerouslySetInnerHTML={{ __html: result }} />


        <ImageComponent images={resultImageUrl ? [resultImageUrl] : []} size={1024} />
      </Flex>

      <Flex gap={'large'} justify="space-between">
        <Select
          defaultValue={prompts_list[0].label}
          style={{ width: '100%' }}
          options={prompts_list}
          onChange={(value) => {
            // request 
            console.log(value);
            if (value === "None" || value === "") {
              return;
            }

            // if value is number
            if (!isNaN(Number(value))) {
              request_system(Number(value))();
              return;
            }

            // set message
            form.setFieldValue("msg", value);
            setMsg(value);
          }}
        />
      </Flex>

      <Form
        name="basic"
        form={form}
        wrapperCol={{ span: 24 }}
        // style={{ maxWidth: 600 }}
        className="form"
        onFinish={(_) => {
          to_request("");
        }}
      >
        <Form.Item<Fields>
          name="msg"
          wrapperCol={{ span: 24 }}
        >
          <TextArea
            ref={inputRef}
            value={msg}
            rows={4}
            // onPaste={(e) => {
            //   // text only
            //   // console.debug("onPaste" + msg);

            //   // 通常通りのペーストを行う
            //   setMsg((prev) => prev + e.clipboardData.getData("text"));
            // }}
            onPasteCapture={(e) => {
              // image only
              if (!e.clipboardData.files.length) {
                return
              }
              e.preventDefault();
              // upload file
              const files = e.clipboardData.files;
              // console.debug("files: ");
              console.debug(files);

              // get image file
              const file = files[0];
              if (file) {
                const base_image = resizeImageAndConvertToBase64(file, MAX_WIDTH, MAX_HEIGHT);
                base_image.then((base64) => {
                  setImageUrl(base64);
                  setIsUpload(false);
                });
              }
            }}
            // onChange={(e) => {
            //   setMsg(e.currentTarget.value)
            // }}
            placeholder="Enter a msg..."
          />
        </Form.Item>

        <Flex gap={"large"}>
          <Row>
            <Col>
              <ImageComponent images={imageUrl ? [imageUrl] : []} size={200} />

              <Flex wrap>
                <ImageComponent images={imageUrls ? imageUrls : []} size={58} />
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
}

// imageがあれば、表示するコンポネント
const ImageComponent = ({ images, size }: { images: string[], size: number }) => {
  return (
    images.map((image, index) => (
      <Image
        key={index}
        width={size}
        src={image}
      />
    ))
  );
}

const resizeImageAndConvertToBase64 = (file: File, maxWidth: number, maxHeight: number): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (event) => {
      const img: HTMLImageElement = document.createElement("img");
      img.onload = () => {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');

        if (!ctx) {
          reject(new Error('Failed to get canvas context'));
          return;
        }

        // 元の画像サイズを取得
        const originalWidth = img.width;
        const originalHeight = img.height;

        // リサイズするサイズを計算
        let newWidth = originalWidth;
        let newHeight = originalHeight;

        if (originalWidth > maxWidth || originalHeight > maxHeight) {
          const widthRatio = maxWidth / originalWidth;
          const heightRatio = maxHeight / originalHeight;
          const bestRatio = Math.min(widthRatio, heightRatio);

          newWidth = originalWidth * bestRatio;
          newHeight = originalHeight * bestRatio;
        }

        canvas.width = newWidth;
        canvas.height = newHeight;
        console.debug("newWidth: ", newWidth, "newHeight: ", newHeight);


        ctx.drawImage(img, 0, 0, newWidth, newHeight);
        const dataUrl = canvas.toDataURL('image/png');
        // console.debug("dataUrl: ", dataUrl);

        resolve(dataUrl);
      };
      img.src = event.target!.result as string;
    };
    reader.onerror = (error) => {
      reject(new Error('Failed to read file: ' + error));
    };

    reader.readAsDataURL(file);
  });
};

export default App;
