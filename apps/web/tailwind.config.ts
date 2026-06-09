import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{js,ts,jsx,tsx,mdx}"],
  theme: {
    extend: {
      colors: {
        rh: {
          green: "var(--rh-green)",
          "green-hover": "var(--rh-green-hover)",
          "green-bright": "var(--rh-green-bright)",
          accent: "var(--rh-accent)",
          "accent-hover": "var(--rh-accent-hover)",
          canvas: "var(--rh-canvas)",
          surface: "var(--rh-surface)",
          "surface-2": "var(--rh-surface-2)",
          border: "var(--rh-border)",
          ink: "var(--rh-ink)",
          muted: "var(--rh-muted)",
          "on-green": "var(--rh-on-green)",
          danger: "var(--rh-danger)",
          warning: "var(--rh-warning)",
        },
      },
      fontFamily: {
        sans: [
          "Inter",
          "system-ui",
          "-apple-system",
          "Segoe UI",
          "Roboto",
          "sans-serif",
        ],
        mono: ["ui-monospace", "SFMono-Regular", "Menlo", "Consolas", "monospace"],
      },
      boxShadow: {
        rh: "0 4px 24px rgba(45, 106, 79, 0.12)",
      },
      animation: {
        "pulse-green": "pulseGreen 2s ease-in-out infinite",
        "fade-up": "fadeUp 0.4s ease-out",
      },
      keyframes: {
        pulseGreen: {
          "0%, 100%": { boxShadow: "0 0 0 0 rgba(45, 106, 79, 0.25)" },
          "50%": { boxShadow: "0 0 0 8px rgba(45, 106, 79, 0)" },
        },
        fadeUp: {
          "0%": { opacity: "0", transform: "translateY(8px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
      },
    },
  },
  plugins: [],
};
export default config;
