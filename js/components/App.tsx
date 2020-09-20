import React from "react";
import { Button } from "@blueprintjs/core";
import { IconNames } from "@blueprintjs/icons";

export const App: React.FC = () => {
  return (
    <>
      <h1>Zuse</h1>
      <div>
        <Button icon={IconNames.THUMBS_UP}>Hello</Button>
      </div>
    </>
  );
};
