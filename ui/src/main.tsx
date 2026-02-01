import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import DebugErrorBoundary from "./components/DebugErrorBoundary";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <DebugErrorBoundary>
      <App />
    </DebugErrorBoundary>
  </React.StrictMode>,
);
