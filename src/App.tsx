import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { Flex, Space, Row, Col, Button, Image, Form, Input, } from "antd";
const { TextArea } = Input;

// Voice API
import "regenerator-runtime/runtime";
import SpeechRecognition, { useSpeechRecognition } from 'react-speech-recognition';

type Fields = {
  b?: number;
  msg?: string;
};

function App() {
  const [form] = Form.useForm();
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [imageUrls, setImageUrls] = useState<string[]>([]);
  const [isUpload, setIsUpload] = useState<boolean>(false);



  const StatusNotSupport = "❌ Browser doesn't support speech recognition."
  const StatusAvailable = "❌ Microphone function is off, access to microphone is required."

  const StatusNone = ""
  const StatusListen = "🎧 Listening..."
  const StatusStop = "🎧 Stoped listening."
  const StatusStart = "🎧 Start listening."
  const StatusThinking = "🤖 Thinking..."
  const StatusModelLow = "🤖 Switch to model 3.5/sonnet."
  const StatusModelHigh = "🤖 Switch to model 4.0/opus."
  const StatusAIChatGPT = "🤖 Switch to ChatGPT."
  const StatusAIClaude = "🤖 Switch to Claude."
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
  const [AI, setAI] = useState<number>(Number);
  const [status, setStatus] = useState(StatusModelHigh);

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
        switch_request(reqest);
      }
    }
  }, [transcript]);


  const switch_request = async (req: string) => {
    if (AI == 0) {
      gpt_request(req)
    } else {
      claude_request(req)
    }
  }

  const claude_request = async (req: string) => {
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

    invoke("claude_request", { b: model, msg: _msg })
      .then((res: any) => { // Add type annotation to 'res'
        console.log(res);

        setResult(`${res}`);
      })
      .catch((err) => {
        console.error(`claude_request > ${err}`);

        setStatus(`error: ${err}`);
      })
      .finally(() => {
        reset_all_vers();
        setQuery(`<h2 class="line_wrap">Q: ${_msg}</h2>\n`);
        if (!listening) {
          setStatus(StatusNone);
        }
      });
  }

  // gpt_request Rust Tauri APIを呼び出す
  const gpt_request = (req: string) => {
    let _msg = msg;
    if (req != "") {
      console.debug("req: ", req);

      _msg = req;
    }
    // console.debug(_msg);

    if (_msg === "") {
      setResult("Please enter a msg.");
      return;
    }
    setStatus(StatusThinking);

    let src = "";
    if (imageUrl && !isUpload) {
      src = imageUrl;
      setImageUrls((prev) => [...prev, imageUrl]);

      setIsUpload(true);
      setImageUrl(null);
    }

    invoke("gpt_request", { b: model, msg: _msg, src: src })
      .then((res) => {
        setResult(`${res}`);
      })
      .catch((err) => {
        console.error(`gpt_request > ${err}`);

        setStatus(`error: ${err}`);
      })
      .finally(() => {
        reset_all_vers();
        setQuery(`<h2 class="line_wrap">Q: ${_msg}</h2>\n`);
        if (!listening) {
          setStatus(StatusNone);
        }
      });
  };

  const reset_messages = () => {
    memo();
    invoke("reset_messages");
    setImageUrls([]);
    setStatus(StatusResetMessages);
  };

  // リセット及びクローズとともにメモを作成する
  const memo = () => {
    invoke("memo")
      .then((message) => {
        setResult(`${message}`);
      })
      .catch((err) => {
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
    if (AI != 0) {
      setAI(0);
      setStatus(StatusAIChatGPT);
    } else {
      setAI(1);
      setStatus(StatusAIClaude);
    }
  }

  // Usefull functions
  const reset_all_vers = () => {
    console.log("reset_all_vers");

    resetTranscript();
    setMsg("");
    setImageUrl(null);
    form.setFieldValue("msg", "");

    // 画面のスクロールを最上部に移動
    window.scrollTo(0, 0);

    console.log(msg);

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
        .then((res) => {
          setStatus(`${res}`);
        })
        .catch((err) => {
          console.error(`request_system > ${err}`);

          setStatus(`error: ${err}`);
        });
    }
  }

  return (
    <Flex gap="large" vertical>
      {/* 各ButtonとButtonの間隔を等間隔にし、かつ、最大幅で設置する */}
      <Flex gap={'large'} justify="space-between">
        <Button size="large" onClick={request_system(1)} title="厳格で正確な">&#x1f9d0;</Button>
        <Button size="large" onClick={request_system(2)} title="フレンドリーな">&#x1fae0;</Button>
        <Button size="large" onClick={request_system(3)} title="肯定的な">&#x1f973;</Button>
        <Button size="large" onClick={request_system(4)} title="批判的な">&#x1f608;</Button>
        <Button size="large" onClick={request_system(0)} title="無指示">&#x1fae5;</Button>
      </Flex>

      <Flex gap={'large'} justify="space-between" vertical={false}>
        <Image preview={false} style={{ maxWidth: '128px' }} onClick={reset_messages} src="/delete.png" className="logo reset message" alt="reset message logo" title="reset messages" />
        <Image preview={false} style={{ maxWidth: '128px' }} onClick={switch_model} src="/switch-model.png" className="logo switch model" alt="switch model logo" title="switch set model" />
        <Image preview={false} style={{ maxWidth: '128px' }} onClick={switch_ai} src={AI === 0 ? "/chatgpt-ai.png" : "/claude-ai.png"} className="logo switch ai" alt="switch ai logo" title="switch set ai" />
        <Image preview={false} style={{ maxWidth: '128px' }} onClick={speech} src="/vc.png" className="logo vc" alt="vc logo" title="start/end vc for message" />
      </Flex>

      <Flex wrap vertical={false} gap={'large'} justify="center">
        <div className="line_wrap" dangerouslySetInnerHTML={{ __html: query }} />

        <div className="code-container" dangerouslySetInnerHTML={{ __html: result }} />
      </Flex>

      <Form
        name="basic"
        form={form}
        wrapperCol={{ span: 24 }}
        // style={{ maxWidth: 600 }}
        className="form"
        onFinish={(_) => {
          switch_request("");
        }}
      >
        <Form.Item<Fields>
          name="msg"
          wrapperCol={{ span: 24 }}
        >
          <TextArea
            value={msg}
            rows={4}
            onPaste={(e) => {
              // text only
              // console.log("onPaste" + msg);

              // 通常通りのペーストを行う
              setMsg((prev) => prev + e.clipboardData.getData("text"));
            }}
            onPasteCapture={(e) => {
              // image only
              if (!e.clipboardData.files.length) {
                return
              }
              e.preventDefault();
              // upload file
              const files = e.clipboardData.files;
              // console.log("files: ");
              // console.log(files);

              // get image file
              const file = files[0];
              if (file) {
                const reader = new FileReader();
                reader.onload = (ev) => {
                  setImageUrl(ev.target?.result as string);
                  setIsUpload(false);
                };
                reader.readAsDataURL(file);
              }
            }}
            onChange={(e) => {
              setMsg(e.currentTarget.value)
            }}
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

export default App;
