export const tokens = {
  color: {
    paper: "#f3efe6",
    surface: "#fffbf4",
    ink: "#1c1917",
    muted: "#6f675e",
    line: "#e4ddd0",
    sage: "#2c6b5a",
    clay: "#c45c26",
  },
  radius: {
    card: 24,
    control: 14,
    pill: 999,
  },
  font: {
    sans: '"IBM Plex Sans", ui-sans-serif, system-ui, sans-serif',
    display: '"Fraunces", "Iowan Old Style", Georgia, serif',
  },
} as const;

export type ThemeName = "light" | "dark";
