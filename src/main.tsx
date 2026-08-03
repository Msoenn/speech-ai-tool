import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { applyPalette, cachedPalette } from "./lib/palettes";

// Apply the last-used palette before first paint (settings load async and
// become the authoritative source once available).
applyPalette(cachedPalette());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
