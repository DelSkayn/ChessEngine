module.exports = {
  mode: "jit",
  content: ["src/**/*.rs"],
  theme: {
    extend: {
      width: {
        "1/8": "12.5%"
      },
      height: {
        "1/8": "12.5%"
      }
    }
  },
  variants: {
    extend: {}
  },
  plugins: [require("daisyui")]
};
