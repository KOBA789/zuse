import React from "react";
import ReactDOM from "react-dom";
import { App } from "~/components/App";

async function main() {
  const root = document.createElement("div");
  document.body.appendChild(root);

  ReactDOM.render(React.createElement(App), root);

  await import("../rs/pkg");
}

main()
  .catch(console.error);
