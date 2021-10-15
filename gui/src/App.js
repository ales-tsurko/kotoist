import React, { useState, useEffect, useRef } from "react";
import Editor from "@monaco-editor/react";
import { FaHammer, FaBroom } from "react-icons/fa";

function App() {
  const [code, setCode] = useState("");
  const [consoleOut, setConsoleOut] = useState("console output");
  const didMountRef = useRef(false);
  const editorRef = useRef(null);

  useEffect(() => {
    if (didMountRef.current) {
      // window.external.invoke("SEND_CONSOLE_OUT " + consoleOut);
    } else {
      didMountRef.current = true;
      setCode(window.external.invoke("GET_CODE"));
      setConsoleOut(window.external.invoke("GET_CONSOLE_OUT"));
      window.addEventListener("SEND_CONSOLE_OUT", (e) =>
        setConsoleOut(e.detail)
      );
    }
  }, [consoleOut]);

  const editorDidMount = (editor) => {
    editorRef.current = editor;
  };

  const onChange = (newValue) => {
    setCode(newValue);
    window.external.invoke("SEND_CODE " + newValue);
  };

  const onClearButtonClick = () => {
    setConsoleOut("");
    window.external.invoke("SEND_CONSOLE_OUT ");
  };

  const onBuildButtonClick = () => {
    const selection = editorRef.current
      .getModel()
      .getValueInRange(editorRef.current.getSelection());
    const block = selection.length > 0 ? selection : code;
    window.external.invoke("EVAL_CODE " + block);
    // setConsoleOut(result);
  };

  return (
    <React.Fragment>
      <Editor
        width="100%"
        height="330px"
        defaultLanguage="coffeescript"
        onChange={onChange}
        value={code}
        onMount={editorDidMount}
      />
      <Toolbar onClear={onClearButtonClick} onBuild={onBuildButtonClick} />
      <Console text={consoleOut} />
    </React.Fragment>
  );
}

function Toolbar(props) {
  return (
    <div className="toolbar">
      <button onClick={props.onBuild}>
        <FaHammer />
      </button>
      <button onClick={props.onClear}>
        <FaBroom />
      </button>
    </div>
  );
}

function Console(props) {
  return <div className="console">{props.text}</div>;
}

export default App;
