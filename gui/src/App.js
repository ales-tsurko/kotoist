import React, { useState, useEffect, useRef } from "react";
import AceEditor from "react-ace";
import { FaHammer, FaBroom } from "react-icons/fa";
import "ace-builds/src-noconflict/mode-javascript";
import "ace-builds/src-noconflict/snippets/javascript";

function App() {
  const [code, setCode] = useState("");
  const [selection, setSelection] = useState("");
  const [consoleOut, setConsoleOut] = useState("console output");
  const didMountRef = useRef(false);

  useEffect(() => {
    if (didMountRef.current) {
      window.external.invoke("SEND_CONSOLE_OUT " + consoleOut);
    } else {
      didMountRef.current = true;
      setCode(window.external.invoke("GET_CODE"));
      setConsoleOut(window.external.invoke("GET_CONSOLE_OUT"));
    }
  }, [consoleOut]);

  const onChange = (newValue) => {
    setCode(newValue);
    window.external.invoke("SEND_CODE " + newValue);
  };

  const onSelectionChange = (newValue) => {
    const selectedText = newValue.doc.getTextRange(newValue.getRange());
    setSelection(selectedText);
  };

  const onClearButtonClick = () => {
    setConsoleOut("");
  };

  const onBuildButtonClick = () => {
    const block = selection.length > 0 ? selection : code;
    const result = window.external.invoke("EVAL_CODE " + block);
    setConsoleOut(result);
  };

  return (
    <React.Fragment>
      <AceEditor
        mode="javascript"
        width="100%"
        height="330px"
        onChange={onChange}
        value={code}
        onSelectionChange={onSelectionChange}
        focus={true}
        enableBasicAutocompletion={true}
        enableLiveAutocompletion={true}
        enableSnippets={true}
        editorProps={{ $blockScrolling: true }}
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
