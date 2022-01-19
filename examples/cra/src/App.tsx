import React from "react"
import * as log from "tauri-plugin-log-api"
import "./App.css"
import logo from "./logo.svg"
import tauriCircles from "./tauri.svg"
import tauriWord from "./wordmark.svg"

function App() {
  function onSelect(event: any) {
    //@ts-ignore
    log[event.target.value](`log from ${event.target.value} event`)
  }

  return (
    <div className="App">
      <header className="App-header">
        <div className="inline-logo">
          <img src={tauriCircles} className="App-logo rotate" alt="logo" />
          <img src={tauriWord} className="App-logo smaller" alt="logo" />
        </div>
        <a
          className="App-link"
          href="https://tauri.studio"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn Tauri
        </a>
        <img src={logo} className="App-logo rotate" alt="logo" />
        <a
          className="App-link"
          href="https://reactjs.org"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn React
        </a>
        <p>
          Edit <code>src/App.tsx</code> and save to reload.
        </p>
        <p>
          <select onChange={onSelect}>
            <option value="info">Info</option>
            <option value="trace">Trace</option>
            <option value="debug">Debug</option>
            <option value="warn">Warn</option>
            <option value="error">Error</option>
          </select>
        </p>
      </header>
    </div>
  )
}

export default App
