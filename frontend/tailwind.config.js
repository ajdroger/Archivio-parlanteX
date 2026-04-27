/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Legal tech dark neon theme
        primary: {
          DEFAULT: '#00ff9f',
          50: '#e6fff7',
          100: '#b3ffe6',
          200: '#80ffd5',
          300: '#4dffc4',
          400: '#1affb3',
          500: '#00ff9f',
          600: '#00cc7f',
          700: '#00995f',
          800: '#00663f',
          900: '#003320',
        },
        dark: {
          DEFAULT: '#0a0f1a',
          50: '#1a2332',
          100: '#141b28',
          200: '#0f1420',
          300: '#0a0f1a',
          400: '#060a12',
          500: '#03050a',
        },
      },
    },
  },
  plugins: [],
}
