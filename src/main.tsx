import React from "react";
import ReactDOM from "react-dom/client";
import { ScanPage } from "./features/scan/ScanPage";
import "./style.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode><ScanPage /></React.StrictMode>
);
