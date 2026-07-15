import React from "react";
import "./Button.css"; // 🔥 Import the button's CSS file here

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
}

/**
 * A generic styled button wrapper component that applies application-wide theme styling.
 * It extends standard HTML button attributes, permitting seamless integration of native event 
 * handlers (like onClick) and attributes (like disabled, type) while keeping design consistent.
 */
export function Button({ children, onClick, ...props }: ButtonProps) {
  return (
    <button onClick={onClick} className="custom-button" {...props}>
      {children}
    </button>
  );
}
