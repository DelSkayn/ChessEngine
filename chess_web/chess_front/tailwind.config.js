
module.exports = {
  mode: 'jit',
  purge: [
    "src/**/*.rs"
  ],
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {
        width: {
            '1/8': '12.5%',
        },
        height: {
            '1/8': '12.5%',
        }
    },
  },
  variants: {
    extend: {},
  },
  plugins: [],
}
