import React, { useState } from "react";
import AceEditor from "react-ace";
import { FaHammer, FaBroom } from "react-icons/fa";

function App() {
  const [code, setCode] = useState("");
  const [selection, setSelection] = useState("");
  const [consoleOut, setConsoleOut] = useState("console output");

  const onChange = (newValue) => {
    setCode(newValue);
  };

  const onSelectionChange = (newValue) => {
    const selectedText = newValue.doc.getTextRange(newValue.getRange());
    setSelection(selectedText);
    setConsoleOut(selectedText);
  };

  const onClearButtonClick = () => {
    setConsoleOut("");
  };

  const onBuildButtonClick = () => {
    const block = selection.length > 0 ? selection : code;
    window.external.invoke("SEND_CODE " + block);
  };

  return (
    <React.Fragment>
      <AceEditor
        mode="javascript"
        width="100%"
        height="330px"
        onChange={onChange}
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
  const text = props.text;
  return <div className="console">{text}</div>;
}

export default App;
