import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          "Inter",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "sans-serif",
        ],
        mono: ["JetBrains Mono", "SFMono-Regular", "Consolas", "monospace"],
      },
      colors: {
        border: "hsl(220 13% 86%)",
        panel: "hsl(0 0% 100%)",
        ink: "hsl(222 24% 12%)",
        muted: "hsl(218 11% 45%)",
        line: "hsl(220 14% 92%)",
        accent: "hsl(173 64% 32%)",
        warn: "hsl(37 85% 45%)",
        danger: "hsl(350 68% 48%)",
      },
      boxShadow: {
        soft: "0 1px 3px rgb(15 23 42 / 0.08)",
      },
    },
  },
  plugins: [],
} satisfies Config;
