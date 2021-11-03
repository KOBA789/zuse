import React from "react";
import ReactDOM from "react-dom";
import { App } from "~/components/App";

async function main() {
  const root = document.createElement("div");
  root.id = "root";
  document.body.appendChild(root);

  ReactDOM.render(React.createElement(App), root);
}

main().catch(console.error);
