import React from "react";
import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import "./Terminal.css";

interface TerminalViewProp {
  userName: string;
  hostName: string;
  folderName?: string;
}

/**
 * Terminal emulator view component. Spawns an xterm.js instance in the dashboard view, 
 * allowing text execution commands. Supports click-dragging the top header bar to dynamically 
 * change the layout height (expanding/collapsing the console space).
 */
export const TerminalView: React.FC<TerminalViewProp> = ({
  userName,
  hostName,
  folderName,
}) => {
  const canvasRef = React.useRef<HTMLDivElement>(null);
  const terminalRef = React.useRef<Terminal | null>(null);

  const [height, setHeight] = useState<number>(260);
  const isDragging = useRef<boolean>(false);
  const startY = useRef<number>(0);
  const startHeight = useRef<number>(0);

  // Captures the initial cursor position and container height when the user clicks the top handle bar.
  // Temporarily adjusts body styling to enforce a row-resize cursor and prevent document text selection.
  const handleMouseDown = (e: React.MouseEvent) => {
    isDragging.current = true;
    startY.current = e.clientY;
    startHeight.current = height;
    document.body.style.cursor = "row-resize"; // Visual feedback
    document.body.style.userSelect = "none"; // Prevent text highlighting
  };

  // Listens to mouse movement coordinates on the window object while dragging is active.
  // Recomputes container pixel height (bounded between 120px and 600px) and cleans up drag events on pointer release.
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return;
      // Moving up decreases clientY, increasing the height
      const deltaY = startY.current - e.clientY;
      const newHeight = Math.max(
        120,
        Math.min(600, startHeight.current + deltaY)
      ); // Min 120px, Max 600px
      setHeight(newHeight);
    };

    const handleMouseUp = () => {
      if (!isDragging.current) return;
      isDragging.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, []);

  // Constructs the default prompt label (e.g. "username@hostname foldername ~ % ") based on 
  // currently available system parameters. Used to prefix command line execution entries.
  const buildTerminalPrompt = () => {
    if (userName && hostName && folderName) {
      return `${userName}@${hostName} ${folderName} ~ % `;
    }
    return `${userName}@${hostName} ~ % `;
  };

  // Instantiates a new xterm.js virtual terminal when the canvas container mount completes.
  // Registers prompt text markers and intercepts keystroke inputs to support basic terminal echo behaviors (Return, Backspace, character printing).
  useEffect(() => {
    if (!canvasRef.current) return;

    const terminal = new Terminal();
    terminal.open(canvasRef.current);
    terminalRef.current = terminal;

    terminal.write(buildTerminalPrompt());

    let inputBuffer = "";

    const dataDisposable = terminal.onData((data: string) => {
      const char = data[0];
      if (char === "\r") {
        terminal.write("\r\n");
        inputBuffer = "";
        terminal.write(buildTerminalPrompt());
      } else if (char === "\u007F") {
        if (inputBuffer.length === 0) return;
        inputBuffer = inputBuffer.slice(0, -1);
        terminal.write("\b \b");
      } else {
        inputBuffer += char;
        terminal.write(char);
      }
    });

    return () => {
      dataDisposable.dispose();
      terminal.dispose();
    };
  }, [userName, hostName, folderName]);

  return (
    <div className="terminal-view-container" style={{ height: `${height}px` }}>
      <div className="terminal-resize-handle" onMouseDown={handleMouseDown} />

      <div className="terminal-canvas-wrapper" ref={canvasRef} />
    </div>
  );
};
