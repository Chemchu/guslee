/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: "selector",
  content: ["./src/**/*.{html,js}", "./templates/**/*.{html,js}"],
  theme: {
    colors: {
     'text-color': '#979dac',
     'background-color': '#33415c',
     'primary-color': '#bc6c25',
     'secondary-color': '#bc6c25',
     'accent-color': '#bc6c25',
     'shade-color': '#606c38',
    },
    extend: {
      typography: () => ({
        theme: {
          css: {
            // Light mode colors
            "--tw-prose-body": "#979dac", // Main text
            "--tw-prose-headings": "#979dac", // Headings
            "--tw-prose-lead": "#979dac", // Lead text
            "--tw-prose-links": "#023e7d", // Links
            "--tw-prose-bold": "#979dac", // Bold text
            "--tw-prose-counters": "#00ADB5", // counters
            "--tw-prose-bullets": "#00ADB5", // Bullets
            "--tw-prose-hr": "#cad2c5", // Horizontal rules
            "--tw-prose-quotes": "#023e7d", // Quotes
            "--tw-prose-quote-borders": "#84a98c", // Quote borders
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
