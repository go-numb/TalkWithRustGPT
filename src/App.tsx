import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

// Voice API
import "regenerator-runtime/runtime";
import SpeechRecognition, { useSpeechRecognition } from 'react-speech-recognition';


function App() {
    const StatusListen = "🎧 Listening..."
    const StatusStop = "🎧 Stoped listening."
    const StatusStart = "🎧 Start listening."
    const StatusThinking = "🤖 Thinking..."
    const StatusModel3_5 = "🤖 Switch to model 3.5."
    const StatusModel4_0 = "🤖 Switch to model 4.0."
    const StatusResetMessages = "📝 Done! reset message history."

  const {
    transcript,
    listening,
    resetTranscript,
    browserSupportsSpeechRecognition,
  } = useSpeechRecognition();
  const [msg, setMsg] = useState("");
  const [query, setQuery] = useState("");
  const [result, setResult] = useState("");
  const [model, setModel] = useState(Number);
  const [status, setStatus] = useState("");

  // useEffect 変数監視セクション
  useEffect(() => { // Resultが更新され、Queryが刷新されたら、入力フォームにフォーカス
    // 入力フォームにフォーカス
    const textField = document.getElementById("input-msg") as HTMLInputElement;
    textField?.focus();
  }, [query]);

  if (!browserSupportsSpeechRecognition) {
    return <span>Browser doesn't support speech recognition.</span>;
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
      console.log(`listening: ${listening}, `, transcript);
      setMsg(transcript);
      setStatus(StatusListen);

      if (is_command_enter(transcript)) {
        console.log("command enter");
        resetTranscript();
        gpt_request();
      }
    }
  }, [transcript]);

  // gpt_request Rust Tauri APIを呼び出す
  function gpt_request() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    const _msg = msg;
    console.log(_msg);
    
    if (_msg === "") {
      setResult("Please enter a msg.");
      return;
    }
    setStatus(StatusThinking);

    invoke("gpt_request", { b: model, msg: _msg })
      .then((res) => {
        setResult(`${res}`);
      })
      .catch((err) => {
        setStatus(`error: ${err}`);
      })
      .finally(() => {
        reset_all_vers();
        setQuery(`<h2 class="line_wrap">Q: ${_msg}</h2>\n`);
      });
  };

  function gpt_reset_messages() {
    memo();
    invoke("gpt_reset_messages");
    setStatus(StatusResetMessages);
  };

  // リセット及びクローズとともにメモを作成する
  function memo() {
    invoke("memo")
      .then((message) => {
        setResult(`${message}`);
      })
      .catch((err) => {
        setResult(err);
      });
  };

  function switch_model() {
    if (model != 0) {
      setModel(0);
      setStatus(StatusModel3_5);
    } else {
      setModel(1);
      setStatus(StatusModel4_0);
    }
  }

  // Usefull functions
  function reset_all_vers() {
    resetTranscript();
    setMsg("");
  }

  function is_command_enter(str: string): Boolean {
    const _msg = str;
    if (_msg.endsWith("エンター") || _msg.endsWith("エンター。") || _msg.endsWith("エンター！")) {
      _msg.replace("エンター", "");
      setMsg(_msg);
      return true;
    }

    return false;
  }


  return (
    <div className="container">
      <h1>Welcome to TalkWithGPT!</h1>

      <div className="row">
        <img onClick={gpt_reset_messages} src="/delete.png" className="logo reset message" alt="reset message logo" title="reset messages" />
        <img onClick={switch_model} src="/switch.png" className="logo switch model" alt="switch model logo" title="switch set model" />
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
          gpt_request();
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
