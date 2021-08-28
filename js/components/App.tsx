/** @jsx jsx */

import React, { useRef } from "react";
import { jsx, css } from "@emotion/react";
import {
  Alignment,
  Button,
  Colors,
  FocusStyleManager,
  Navbar,
  NavbarGroup,
} from "@blueprintjs/core";
import { IconNames } from "@blueprintjs/icons";
import { Handler, ZsCad } from "./ZsCad";

FocusStyleManager.onlyShowFocusOnTabs();

const sidePaneStyle = css`
  background-color: ${Colors.WHITE};
  width: 300px;
  padding: 18px 12px;
`;
const SidePane: React.FC = ({ children }) => (
  <div css={sidePaneStyle}>{children}</div>
);

const mainPaneStyle = css`
  flex: 1 1 auto;
`;
const MainPane: React.FC = ({ children }) => (
  <div css={mainPaneStyle}>{children}</div>
);

const sideBySideStyle = css`
  display: flex;
  flex-direction: row;
  align-items: stretch;
  flex: 1 0 auto;
`;
const SideBySide: React.FC = ({ children }) => (
  <div css={sideBySideStyle}>{children}</div>
);

const navbarGroupStyle = css`
  justify-content: center;
`;

const appStyle = css`
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
`;

export const App: React.FC = () => {
  const zsCadRef = useRef(null as Handler | null);
  const handleDownloadClick = () => {
    const blob = new Blob([
      zsCadRef.current!.saveSchematic()
    ], { type : 'application/json' });
    const objectUrl = URL.createObjectURL(blob);;
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
    input.addEventListener("change", async (event) => {
      const data = await (event.target as HTMLInputElement).files![0].text();
      zsCadRef.current!.loadSchematic(data);
      input.remove();
    }, false);
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
    <div css={appStyle}>
      <Navbar>
        <NavbarGroup align={Alignment.LEFT} css={navbarGroupStyle}>
        <Button minimal large icon={IconNames.DOWNLOAD} onClick={handleDownloadClick}/>
        <Button minimal large icon={IconNames.DOCUMENT_OPEN} onClick={handleOpenClick}/>
        </NavbarGroup>
        <NavbarGroup align={Alignment.CENTER} css={navbarGroupStyle}>
          {/*
          <Button minimal large icon={IconNames.SELECT} />
          <Button minimal large icon={IconNames.MINUS} />
          <Button minimal large icon={IconNames.DOT} />
          <Button minimal large icon={IconNames.CIRCLE} />
          <Button minimal large icon={IconNames.KEY_OPTION} />
          <Button minimal large icon={IconNames.SYMBOL_TRIANGLE_UP} />*/}
          <Button minimal large icon={IconNames.PLAY} onClick={handleStartSimulation} />
          <Button minimal large icon={IconNames.STOP} onClick={handleStopSimulation} />
        </NavbarGroup>
      </Navbar>
      <SideBySide>
        <MainPane>
          <ZsCad ref={zsCadRef} />
        </MainPane>
        <SidePane />
      </SideBySide>
    </div>
  );
};
