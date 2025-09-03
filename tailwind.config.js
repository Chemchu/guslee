/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["selector"],
  content: ["./src/**/*.{html,js}", "./templates/**/*.{html,js}"],
  theme: {
    extend: {
      typography: () => ({
        earthy: {
          css: {
            // Light mode colors
            "--tw-prose-body": "#2f3e46", // Charcoal for main text
            "--tw-prose-headings": "#354f52", // Dark slate gray for headings
            "--tw-prose-lead": "#52796f", // Hooker's green for lead text
            "--tw-prose-links": "#52796f", // Hooker's green for links
            "--tw-prose-bold": "#354f52", // Dark slate gray for bold text
            "--tw-prose-counters": "#84a98c", // Cambridge blue for counters
            "--tw-prose-bullets": "#84a98c", // Cambridge blue for bullets
            "--tw-prose-hr": "#cad2c5", // Ash gray for horizontal rules
            "--tw-prose-quotes": "#2f3e46", // Charcoal for quotes
            "--tw-prose-quote-borders": "#84a98c", // Cambridge blue for quote borders
            "--tw-prose-captions": "#52796f", // Hooker's green for captions
            "--tw-prose-code": "#354f52", // Dark slate gray for inline code
            "--tw-prose-pre-code": "#cad2c5", // Ash gray for code in pre blocks
            "--tw-prose-pre-bg": "#2f3e46", // Charcoal background for pre blocks
            "--tw-prose-th-borders": "#cad2c5", // Ash gray for table header borders
            "--tw-prose-td-borders": "#cad2c5", // Ash gray for table cell borders
            // Dark mode (invert) colors
            "--tw-prose-invert-body": "#cad2c5", // Ash gray for main text
            "--tw-prose-invert-headings": "#cad2c5", // Ash gray for headings
            "--tw-prose-invert-lead": "#84a98c", // Cambridge blue for lead text
            "--tw-prose-invert-links": "#84a98c", // Cambridge blue for links
            "--tw-prose-invert-bold": "#cad2c5", // Ash gray for bold text
            "--tw-prose-invert-counters": "#84a98c", // Cambridge blue for counters
            "--tw-prose-invert-bullets": "#84a98c", // Cambridge blue for bullets
            "--tw-prose-invert-hr": "#52796f", // Hooker's green for horizontal rules
            "--tw-prose-invert-quotes": "#cad2c5", // Ash gray for quotes
            "--tw-prose-invert-quote-borders": "#52796f", // Hooker's green for quote borders
            "--tw-prose-invert-captions": "#84a98c", // Cambridge blue for captions
            "--tw-prose-invert-code": "#84a98c", // Cambridge blue for inline code
            "--tw-prose-pre-code": "#cad2c5", // Ash gray for code in pre blocks
            "--tw-prose-pre-bg": "rgba(47, 62, 70, 0.5)", // Semi-transparent charcoal
            "--tw-prose-invert-th-borders": "#52796f", // Hooker's green for table header borders
            "--tw-prose-invert-td-borders": "#84a98c", // Cambridge blue for table cell borders
          },
        },
      }),
    },
  },
  plugins: [],
};
