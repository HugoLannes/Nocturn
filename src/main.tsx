import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { OverlayCardApp } from "./OverlayCardApp";
import "./index.css";

async function bootstrap() {
  // Plain-browser dev mode (`npm run dev` outside Tauri): install IPC mocks
  // before anything touches the Tauri APIs.
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    await import("./dev/mocks");
  }

  const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
  const isOverlayCardWindow = getCurrentWebviewWindow().label.startsWith("overlay-card-");

  ReactDOM.createRoot(document.getElementById("root")!).render(
    isOverlayCardWindow ? <OverlayCardApp /> : <App />,
  );
}

void bootstrap();
