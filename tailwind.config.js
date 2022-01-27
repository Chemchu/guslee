module.exports = {
  content: ['./src/pages/**/*.{js,ts,jsx,tsx}', './src/components/**/*.{js,ts,jsx,tsx}'],
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {
      fontFamily: {
        sans: ['"Open Sans"', '"Public Sans"', 'sans-serif', 'system-ui'],
        mono: ['monospace', 'SFMono-Regular'],
        outfit: ['Outfit', 'cursive'],
        "outfit-light": ['OutfitLight', 'cursive'],
        "outfit-thin": ['OutfitThin', 'cursive']
      },
      backgroundImage: {
        'main': "url('/img/window.jpg')",
      }
    },
  },
  variants: {
    extend: {},
  },
}