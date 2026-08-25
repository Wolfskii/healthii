export const palettes = {
  light: {
    paper: "#f3efe6",
    surface: "#fffbf4",
    sunken: "#ebe4d7",
    ink: "#1c1917",
    muted: "#6f675e",
    line: "#e4ddd0",
    sage: "#2c6b5a",
    sageSoft: "#d9ece4",
    clay: "#c45c26",
    onSage: "#fffbf4",
  },
  dark: {
    paper: "#141210",
    surface: "#1e1b18",
    sunken: "#2a2622",
    ink: "#f5f0e8",
    muted: "#b7aea3",
    line: "#3a342e",
    sage: "#7dcfb6",
    sageSoft: "#1b3a32",
    clay: "#e08a54",
    onSage: "#141210",
  },
} as const;

export const tokens = {
  color: palettes.light,
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

export type ThemeName = keyof typeof palettes;
export type ThemePreference = ThemeName | "system";
export type ColorPalette = (typeof palettes)[ThemeName];
