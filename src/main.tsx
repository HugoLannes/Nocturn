import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import App from "./App";
import { OverlayCardApp } from "./OverlayCardApp";
import "./index.css";

const currentWindow = getCurrentWebviewWindow();
const isOverlayCardWindow = currentWindow.label.startsWith("overlay-card-");

ReactDOM.createRoot(document.getElementById("root")!).render(
  isOverlayCardWindow ? <OverlayCardApp /> : <App />,
);
