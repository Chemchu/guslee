module.exports = {
  content: ['./src/pages/**/*.{js,ts,jsx,tsx}', './src/components/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'media', // or 'media' or 'class'
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
      },
      animation: {
            'gradient-x':'gradient-x 8s ease-in-out infinite',
            'gradient-y':'gradient-y 8s ease-in-out infinite',
            'gradient-xy':'gradient-xy 8s ease-in-out infinite',
        },
      keyframes: {
        'gradient-y': {
          '0%, 100%': {
              'background-size':'400% 400%',
              'background-position': 'center top'
          },
          '50%': {
              'background-size':'200% 200%',
              'background-position': 'center center'
          }
        },
        'gradient-x': {
          '0%, 100%': {
              'background-size':'200% 200%',
              'background-position': 'left center'
          },
          '50%': {
              'background-size':'200% 200%',
              'background-position': 'right center'
          }
        },
        'gradient-xy': {
          '0%, 100%': {
              'background-size':'400% 400%',
              'background-position': 'left center'
          },
          '50%': {
              'background-size':'200% 200%',
              'background-position': 'right center'
          }
        }
      }
    },
  },
  variants: {
    extend: {},
  },
}