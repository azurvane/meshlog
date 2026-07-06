import React from "react";
import "./Button.css"; // 🔥 Import the button's CSS file here

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
}

export function Button({ children, onClick, ...props }: ButtonProps) {
  return (
    <button onClick={onClick} className="custom-button" {...props}>
      {children}
    </button>
  );
}
