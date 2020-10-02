/** @jsx jsx */

import React from "react";
import { jsx, css } from "@emotion/core";
import {
  Alignment,
  Button,
  Colors,
  FocusStyleManager,
  Navbar,
  NavbarGroup,
} from "@blueprintjs/core";
import { IconNames } from "@blueprintjs/icons";
import { ZsCad } from "./ZsCad";

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
  return (
    <div css={appStyle}>
      <Navbar>
        <NavbarGroup align={Alignment.CENTER} css={navbarGroupStyle}>
          <Button minimal large icon={IconNames.SELECT} />
          <Button minimal large icon={IconNames.MINUS} />
          <Button minimal large icon={IconNames.DOT} />
          <Button minimal large icon={IconNames.CIRCLE} />
          <Button minimal large icon={IconNames.KEY_OPTION} />
          <Button minimal large icon={IconNames.SYMBOL_TRIANGLE_UP} />
        </NavbarGroup>
      </Navbar>
      <SideBySide>
        <MainPane>
          <ZsCad />
        </MainPane>
        <SidePane />
      </SideBySide>
    </div>
  );
};
