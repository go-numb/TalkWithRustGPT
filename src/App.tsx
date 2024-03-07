import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

// Voice API
import "regenerator-runtime/runtime";
import SpeechRecognition, { useSpeechRecognition } from 'react-speech-recognition';


function App() {
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
  const [query, setQuery] = useState("");
  const [result, setResult] = useState("");
  const [model, setModel] = useState(Number);
  const [AI, setAI] = useState(Number);
  const [status, setStatus] = useState("");

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
    const textField = document.getElementById("input-msg") as HTMLInputElement;
    textField?.focus();
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
      console.debug(`listening: ${listening}, `, transcript);
      setMsg(transcript);
      setStatus(StatusListen);

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
  function gpt_request(req: string) {
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

    invoke("gpt_stream_request", { b: model, msg: _msg })
      .then((res) => {
        setResult(`${res}`);
      })
      .catch((err) => {
        console.error(`gpt_stream_request > ${err}`);

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

  function reset_messages() {
    memo();
    invoke("reset_messages");
    setStatus(StatusResetMessages);
  };

  // リセット及びクローズとともにメモを作成する
  function memo() {
    invoke("memo")
      .then((message) => {
        setResult(`${message}`);
      })
      .catch((err) => {
        console.error(`memo > ${err}`);
        setResult(err);
      });
  };

  function switch_model() {
    if (model != 0) {
      setModel(0);
      setStatus(StatusModelLow);
    } else {
      setModel(1);
      setStatus(StatusModelHigh);
    }
  }

  function switch_ai() {
    if (AI != 0) {
      setAI(0);
      setStatus(StatusAIChatGPT);
    } else {
      setAI(1);
      setStatus(StatusAIClaude);
    }
  }

  // Usefull functions
  function reset_all_vers() {
    resetTranscript();
    setMsg("");
  }

  function is_command_enter(str: string): [Boolean, string] {
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

  function request_system(num: number) {
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
    <div className="container">
      <div className="row" style={{
        marginBottom: "1rem",
      }}>
        <div style={{
          position: "absolute",
          top: "1rem",
          textAlign: "center",
          display: "flex",
          justifyContent: "space-between",
          width: "100%",
        }}>
          {/* 各ButtonとButtonの間隔を等間隔にし、かつ、最大幅で設置する */}
          <button style={{ flexGrow: 0 }} onClick={request_system(1)} title="厳格で正確な">&#x1f9d0;</button>
          <button style={{ flexGrow: 0 }} onClick={request_system(2)} title="フレンドリーな">&#x1fae0;</button>
          <button style={{ flexGrow: 0 }} onClick={request_system(3)} title="肯定的な">&#x1f973;</button>
          <button style={{ flexGrow: 0 }} onClick={request_system(4)} title="批判的な">&#x1f608;</button>
          <button style={{ flexGrow: 0 }} onClick={request_system(0)} title="無指示">&#x1fae5;</button>
        </div>
      </div>

      <div className="row">
        <img onClick={reset_messages} src="/delete.png" className="logo reset message" alt="reset message logo" title="reset messages" />
        <img onClick={switch_model} src="/switch-model.png" className="logo switch model" alt="switch model logo" title="switch set model" />
        <img onClick={switch_ai} src={AI === 0 ? "/chatgpt-ai.png" : "/claude-ai.png"} className="logo switch ai" alt="switch ai logo" title="switch set ai" />
        <img onClick={speech} src="/vc.png" className="logo vc" alt="vc logo" title="start/end vc for message" />
      </div>

      <div style={{ textAlign: "left" }}>
        <div dangerouslySetInnerHTML={{ __html: query }} />
      </div>
      <div style={{ textAlign: "left" }} className="word-break" >
        <div dangerouslySetInnerHTML={{ __html: result }} />
      </div>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          switch_request("");
        }}
      >
        <textarea
          id="input-msg"
          value={msg}
          rows={5}
          cols={60}
          onChange={(e) => setMsg(e.currentTarget.value)}
          placeholder="Enter a msg..."
        />
        <button type="submit">send</button>
      </form>
      <div style={{ textAlign: "center" }} className="word-break" >
        {status}
      </div>
    </div>
  );
}

export default App;
