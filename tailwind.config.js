/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ['selector'],
  content: ["./src/**/*.{html,js}", "./templates/**/*.{html,js}"],
  theme: {
    extend: {
      animation: {
        wiggle: 'wiggle 1s ease-in-out infinite',
      }
    }
  },
  plugins: [],
}
