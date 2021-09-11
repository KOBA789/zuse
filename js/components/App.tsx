/** @jsx jsx */

import React, { useRef, useState } from "react";
import { jsx, css } from "@emotion/react";
import {
  Alignment,
  Button,
  Classes,
  Dialog,
  FocusStyleManager,
  Icon,
  Navbar,
} from "@blueprintjs/core";
import { IconNames } from "@blueprintjs/icons";
import { Handler, ZsCad } from "./ZsCad";

FocusStyleManager.onlyShowFocusOnTabs();

const appStyle = css`
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
`;

export const App: React.FC = () => {
  const [isHelpOpen, setIsHelpOpen] = useState(false);
  const zsCadRef = useRef(null as Handler | null);
  const handleDownloadClick = () => {
    const blob = new Blob([zsCadRef.current!.saveSchematic()], {
      type: "application/json",
    });
    const objectUrl = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.download = "schematic.zse";
    a.href = objectUrl;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(objectUrl);
  };
  const handleOpenClick = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".zse";
    input.addEventListener(
      "change",
      async (event) => {
        const data = await (event.target as HTMLInputElement).files![0].text();
        zsCadRef.current!.loadSchematic(data);
        input.remove();
      },
      false
    );
    document.body.appendChild(input);
    input.click();
  };
  const handleStartSimulation = () => {
    zsCadRef.current!.startSimulation();
  };
  const handleStopSimulation = () => {
    zsCadRef.current!.stopSimulation();
  };
  return (
    <div className="flex flex-col w-full h-full">
      <Navbar>
        <Navbar.Group align={Alignment.LEFT}>
          <Navbar.Heading>Zuse</Navbar.Heading>
          <Button
            minimal
            large
            icon={IconNames.DOWNLOAD}
            onClick={handleDownloadClick}
          />
          <Button
            minimal
            large
            icon={IconNames.DOCUMENT_OPEN}
            onClick={handleOpenClick}
          />
        </Navbar.Group>
        <Navbar.Group align={Alignment.CENTER} className="justify-center">
          <Button
            minimal
            large
            icon={IconNames.PLAY}
            onClick={handleStartSimulation}
          />
          <Button
            minimal
            large
            icon={IconNames.STOP}
            onClick={handleStopSimulation}
          />
          <Navbar.Divider />
          <Button
            minimal
            large
            icon={IconNames.HELP}
            onClick={() => setIsHelpOpen(true)}
          />
          <Dialog
            canEscapeKeyClose
            canOutsideClickClose
            hasBackdrop
            isCloseButtonShown
            title="Help"
            icon={IconNames.HELP}
            isOpen={isHelpOpen}
            onClose={() => setIsHelpOpen(false)}
          >
            <div className={Classes.DIALOG_BODY}>
              <dl>
                <dt>Key W</dt>
                <dd>
                  <strong>W</strong>iring
                </dd>
                <dt>Key C</dt>
                <dd>
                  Relay <strong>C</strong>oil
                </dd>
                <dt>Key S</dt>
                <dd>
                  <strong>S</strong>witch
                </dd>
                <dt>Key P</dt>
                <dd>
                  <strong>P</strong>ower Source
                </dd>
                <dt>Key R</dt>
                <dd>
                  <strong>R</strong>otate switch
                </dd>
                <dt>Key Y</dt>
                <dd>Flip switch horizontally</dd>
                <dt>Key D</dt>
                <dd>
                  <strong>D</strong>elete wires or components
                </dd>
                <dt>Double-click component</dt>
                <dd>Change ID</dd>
              </dl>
            </div>
          </Dialog>
        </Navbar.Group>
      </Navbar>
      <ZsCad ref={zsCadRef} />
      <div className="p-1 bg-white text-right">
        Made by <a href="https://koba789.com">KOBA789</a>
      </div>
    </div>
  );
};
