/** @jsx jsx */
import { css, jsx } from "@emotion/core";
import React, { useEffect, useRef } from "react";
import type { ZsSchIo } from "../../rs/pkg";

const zsCadStyle = css`
  display: block;
  width: 100%;
  height: 100%;
`;

export const ZsCad: React.FC = () => {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const webgl = canvas.current!.getContext("webgl")!;
    let isUnmounted = false;
    let io: ZsSchIo | null = null;
    import("../../rs/pkg").then(({ ZsSch, ZsSchIo }) => {
      if (isUnmounted) {
        return;
      }
      io = new ZsSchIo();
      const zsSch = new ZsSch(webgl);
      const loop = () => {
        if (isUnmounted) {
          zsSch.free();
          return;
        }
        const width = canvas.current!.clientWidth;
        const height = canvas.current!.clientHeight;
        canvas.current!.width = width;
        canvas.current!.height = height;
        zsSch.set_frame_size(width, height);
        zsSch.new_frame(io!);
        zsSch.draw();
        requestAnimationFrame(loop);
      };
      requestAnimationFrame(loop);
    });
    const currentCanvas = canvas.current!;
    const onWheel = function (this: HTMLCanvasElement, e: WheelEvent) {
      e.preventDefault();
      if (!io) {
        return;
      }
      if (e.ctrlKey) {
        io.pinch += e.deltaY;
      } else {
        io.wheelX += e.deltaX;
        io.wheelY += e.deltaY;
      }
    };
    const onMouseMove = function (this: HTMLCanvasElement, e: MouseEvent) {
      e.preventDefault();
      if (!io) {
        return;
      }
      const rect = canvas.current!.getBoundingClientRect();
      io.mouseX = e.clientX - rect.left;
      io.mouseY = e.clientY - rect.top;
    };
    const onKeyDown = function (this: HTMLCanvasElement, e: KeyboardEvent) {
      e.preventDefault();
      if (!io) {
        return;
      }
      io.keydown = e.key;
    };
    currentCanvas.addEventListener("wheel", onWheel);
    currentCanvas.addEventListener("mousemove", onMouseMove);
    currentCanvas.addEventListener("keydown", onKeyDown);
    return () => {
      isUnmounted = true;
      currentCanvas.removeEventListener("wheel", onWheel);
      currentCanvas.removeEventListener("mousemove", onMouseMove);
      currentCanvas.removeEventListener("keydown", onKeyDown);
    };
  }, []);
  return <canvas ref={canvas} css={zsCadStyle} tabIndex={0} />;
};
