import React, {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import init, {
  Cad,
  ComponentMetadata,
  GlowBackend,
  Io,
} from "@crate/zuse-rs/pkg";

export type Handler = {
  saveSchematic: () => string;
  loadSchematic: (zse: string) => void;
  startSimulation: () => void;
  stopSimulation: () => void;
};

export type ZsCadProps = {};

export const ZsCad = forwardRef<Handler, ZsCadProps>((_, ref) => {
  const wrapper = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const [zsSch, setZsSch] = useState(null as Cad | null);
  useImperativeHandle(ref, () => {
    return {
      saveSchematic: () => {
        if (zsSch === null) {
          throw new Error("Cad is not initialized");
        }
        return zsSch.save_schematic();
      },
      loadSchematic: (zse) => {
        if (zsSch === null) {
          throw new Error("Cad is not initialized");
        }
        zsSch.load_schematic(zse);
      },
      startSimulation: () => {
        if (zsSch === null) {
          throw new Error("Cad is not initialized");
        }
        zsSch.start_simulation();
      },
      stopSimulation: () => {
        if (zsSch === null) {
          throw new Error("Cad is not initialized");
        }
        zsSch.stop_simulation();
      },
    };
  });
  useEffect(() => {
    const webgl = canvas.current!.getContext("webgl2")!;
    let isUnmounted = false;
    let io: Io | null = null;
    init().then(() => {
      if (isUnmounted) {
        return;
      }
      const backend = new GlowBackend(webgl);
      io = new Io();
      const zsSch = new Cad(backend);
      setZsSch(zsSch);
      const loop = () => {
        if (isUnmounted) {
          zsSch.free();
          return;
        }
        const width = wrapper.current!.clientWidth;
        const height = wrapper.current!.clientHeight;
        canvas.current!.width = width * window.devicePixelRatio;
        canvas.current!.height = height * window.devicePixelRatio;
        //canvas.current!.style.width = `${width}px`;
        //canvas.current!.style.height = `${height}px`;
        io!.setScreenSize(width, height, window.devicePixelRatio);
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
    const onMouseDown = function (this: HTMLCanvasElement, _e: MouseEvent) {
      this.focus();
      if (!io) {
        return;
      }
      //io.push_keydown(e.button);
    };
    const onMouseUp = function (this: HTMLCanvasElement, _e: MouseEvent) {
      if (!io) {
        return;
      }
      //io.buttons = e.buttons;
    };
    const onClick = function (this: HTMLCanvasElement, e: MouseEvent) {
      if (!io) {
        return;
      }
      io.pushClick(e.button);
    };
    const onDoubleClick = function (this: HTMLCanvasElement, e: MouseEvent) {
      if (!io) {
        return;
      }
      io.pushDoubleClick(e.button);
    };
    const onKeyDown = function (this: HTMLCanvasElement, e: KeyboardEvent) {
      e.preventDefault();
      if (!io) {
        return;
      }
      io.pushKeydown(e.key);
    };
    currentCanvas.addEventListener("wheel", onWheel);
    currentCanvas.addEventListener("mousemove", onMouseMove);
    currentCanvas.addEventListener("mousedown", onMouseDown);
    currentCanvas.addEventListener("mouseup", onMouseUp);
    currentCanvas.addEventListener("click", onClick);
    currentCanvas.addEventListener("dblclick", onDoubleClick);
    currentCanvas.addEventListener("keydown", onKeyDown);
    return () => {
      isUnmounted = true;
      currentCanvas.removeEventListener("wheel", onWheel);
      currentCanvas.removeEventListener("mousemove", onMouseMove);
      currentCanvas.removeEventListener("mousedown", onMouseDown);
      currentCanvas.removeEventListener("mouseup", onMouseUp);
      currentCanvas.removeEventListener("click", onClick);
      currentCanvas.removeEventListener("keydown", onKeyDown);
    };
  }, []);
  return (
    <div ref={wrapper} className="flex-grow overflow-hidden w-full h-full">
      <canvas ref={canvas} tabIndex={0} className="block" />
    </div>
  );
});
ZsCad.displayName = "ZsCad";
