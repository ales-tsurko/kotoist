import React, { useState } from "react";
import AceEditor from "react-ace";
import { FaHammer, FaBroom } from "react-icons/fa";

function App() {
  const [code, setCode] = useState("// type your code...");
  const [consoleOut, setConsoleOut] = useState("console output");

  const onChange = (newValue) => {
    console.log("onChange", newValue);
    setCode(newValue);
  };

  const onClearButtonClick = () => {
    setConsoleOut("");
  };

  const onBuildButtonClick = () => {
    console.log("build");
  };

  return (
    <React.Fragment>
      <AceEditor
        mode="javascript"
        width="100%"
        height="330px"
        onChange={onChange}
        value={code}
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
  return (
    <div contentEditable={true} className="console">
      {text}
    </div>
  );
}

export default App;
