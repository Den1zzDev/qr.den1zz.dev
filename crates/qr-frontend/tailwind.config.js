const path = require('path');

module.exports = {
  content: [
    path.join(__dirname, "src/**/*.rs"),
    path.join(__dirname, "src/bin/**/*.rs"),
    "./crates/qr-frontend/src/**/*.rs",
    "./src/**/*.rs",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        mono: ['"Maple Mono NF"', '"Maple Mono"', 'ui-monospace', 'SF Mono', 'Menlo', 'Consolas', 'monospace'],
        sans: ['"Maple Mono NF"', '"Maple Mono"', 'ui-monospace', 'SF Mono', 'Menlo', 'Consolas', 'monospace'],
      },
      colors: {
        bg: '#000000',
        'bg-elevated': '#080808',
        panel: '#0a0a0a',
        'panel-border': 'rgba(255, 255, 255, 0.12)',
        accent: '#00f0ff',
        'accent-dim': '#008b99',
        'accent-glow': 'rgba(0, 240, 255, 0.25)',
        text: '#ffffff',
        'text-dim': '#a0a0a0',
        'text-faint': '#555555',
      }
    }
  }
};
