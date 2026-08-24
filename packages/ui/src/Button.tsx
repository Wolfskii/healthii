import type { ButtonHTMLAttributes } from "react";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "secondary" | "icon";
};

export function Button({ variant = "primary", className = "", ...props }: ButtonProps) {
  const extras = variant === "primary" ? "" : variant;
  return <button className={`hii-btn ${extras} ${className}`.trim()} {...props} />;
}
