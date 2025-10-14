/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: "selector",
  content: {
    files: ["./src/**/*.{html,js}", "./templates/**/*.{html,js}"],
    extract: {
      rs: (content) => content.match(/class\s*=\s*"([^"]*)"/g) || []
    }
  },
  theme: {
    colors: {
     'text-color': '#DBDFE5',
     'background-color': '#1E242C',
     'primary-color': '#F58A07',
     'secondary-color': '#bc6c25',
     'accent-color': '#bc6c25',
     'shade-color': '#323C49',
    },
    extend: {
      typography: () => ({
        theme: {
          css: {
            h1: {
              fontSize: "2.5em"
            },
            h2: {
              fontSize: "2em"
            },
            "--tw-prose-headings": "#DBDFE5", // Headings
            "--tw-prose-body": "#DBDFE5", // Main text
            "--tw-prose-lead": "#DBDFE5", // Lead text
            "--tw-prose-bold": "#DBDFE5", // Bold text
            "--tw-prose-links": "#F58A07", // Links
            "--tw-prose-counters": "#F58A07", // counters
            "--tw-prose-bullets": "#F58A07", // Bullets
            "--tw-prose-hr": "#F58A07", // Horizontal rules
            "--tw-prose-quotes": "#DBDFE5", // Quotes
            "--tw-prose-quote-borders": "#F58A07", // Quote borders
            "--tw-prose-captions": "#52796f", // Captions
            "--tw-prose-code": "#354f52", // Inline code
            "--tw-prose-pre-code": "#cad2c5", // Code in pre blocks
            "--tw-prose-pre-bg": "#2f3e46", // Pre blocks
            "--tw-prose-th-borders": "#cad2c5", // Table header borders
            "--tw-prose-td-borders": "#cad2c5", // Table cell borders
          },
        },
      }),
    },
  },
  plugins: [],
};
