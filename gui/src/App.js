import React, { useState, useEffect, useRef } from "react";
import Editor from "@monaco-editor/react";
import { FaHammer, FaBroom, FaTh } from "react-icons/fa";

function App() {
  const [code, setCode] = useState("");
  const [consoleOut, setConsoleOut] = useState("console output");
  const didMountRef = useRef(false);
  const editorRef = useRef(null);
  const [padsVisible, setPadsVisible] = useState(false);
  const [currentPadSelection, setCurrentPadSelection] = useState({});

  useEffect(() => {
    if (didMountRef.current) {
      // window.external.invoke("SEND_CONSOLE_OUT " + consoleOut);
    } else {
      didMountRef.current = true;
      setCode(window.external.invoke("GET_CODE"));
      setCurrentPadSelection(
        JSON.parse(window.external.invoke("GET_SELECTED_PAD"))
      );
      setConsoleOut(window.external.invoke("GET_CONSOLE_OUT"));
      window.addEventListener("SEND_CONSOLE_OUT", (e) =>
        setConsoleOut(e.detail)
      );
    }
  }, [consoleOut]);

  const editorDidMount = (editor) => {
    editorRef.current = editor;
  };

  const onPadsSelectionChange = (value) => {
    setCurrentPadSelection(value);
    window.external.invoke(`SELECT_PAD ${JSON.stringify(value)}`);
    setCode(window.external.invoke("GET_CODE"));
  };

  const onChange = (newValue) => {
    setCode(newValue);
    window.external.invoke("SEND_CODE " + newValue);
  };

  const onTogglePads = () => {
    setPadsVisible(!padsVisible);
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
  };

  return (
    <React.Fragment>
      <div className="editor-container">
        <Pads visible={padsVisible} onSelectionChange={onPadsSelectionChange} />
        <Editor
          width="100%"
          height="330px"
          defaultLanguage="coffeescript"
          onChange={onChange}
          value={code}
          onMount={editorDidMount}
          options={{ minimap: { enabled: false }, rulers: [80], tabSize: 2 }}
        />
      </div>
      <Toolbar
        onTogglePads={onTogglePads}
        onClear={onClearButtonClick}
        onBuild={onBuildButtonClick}
        padSelection={currentPadSelection}
      />
      <Console text={consoleOut} />
    </React.Fragment>
  );
}

function Pads(props) {
  const [selection, setSelection] = useState(0);

  const onSelectionChange = (value) => {
    setSelection(value.number);
    props.onSelectionChange(value);
  };

  const pads = Array(128)
    .fill(0)
    .map((_, n) => (
      <Pad
        key={n.toString()}
        number={n}
        onSelectionChange={onSelectionChange}
        selected={selection === n}
      />
    ));

  return (
    <div className={`pads ${props.visible ? "pads-visible" : ""}`}>{pads}</div>
  );
}

function Pad(props) {
  const [isMouseDown, setIsMouseDown] = useState(false);
  const [isHover, setIsHover] = useState(false);
  const defaultValue = `snippet ${props.number + 1}`;
  const [name, setName] = useState(defaultValue);
  const pitches = [
    "C",
    "C#",
    "D",
    "D#",
    "E",
    "F",
    "F#",
    "G",
    "G#",
    "A",
    "A#",
    "B",
  ];
  const noteName =
    pitches[props.number % pitches.length] + Math.floor(props.number / 12);

  const onMouseDown = () => {
    setIsMouseDown(true);
  };

  const onMouseUp = () => {
    setIsMouseDown(false);
  };

  const onMouseEnter = () => {
    setIsHover(true);
  };

  const onMouseLeave = () => {
    setIsHover(false);
  };

  return (
    <div className="pad">
      <div
        className={`pad-button ${
          isHover && !isMouseDown ? "pad-button-hover" : ""
        }`}
        onMouseDown={onMouseDown}
        onMouseUp={onMouseUp}
        onMouseEnter={onMouseEnter}
        onMouseLeave={onMouseLeave}
      >
        {noteName}
      </div>
      <input
        type="text"
        className={`pad-selection ${props.selected ? "pad-selected" : ""}`}
        onClick={() =>
          props.onSelectionChange({ number: props.number, noteName, name })
        }
        value={props.padName || defaultValue}
        maxLength={20}
        onChange={(event) => setName(event.target.value)}
      />
    </div>
  );
}

function Toolbar(props) {
  const selectionText = props.padSelection.noteName
    ? `${props.padSelection.number + 1} | \
  ${props.padSelection.noteName} | ${props.padSelection.name}`
    : "";
  return (
    <div className="toolbar">
      <label className="toolbar-selection-text">{selectionText}</label>
      <div className="toolbar-buttons-container">
        <button onClick={props.onTogglePads}>
          <FaTh />
        </button>
        <button onClick={props.onBuild}>
          <FaHammer />
        </button>
        <button onClick={props.onClear}>
          <FaBroom />
        </button>
      </div>
    </div>
  );
}

function Console(props) {
  return (
    <div className="console" dangerouslySetInnerHTML={{ __html: props.text }} />
  );
}

export default App;
