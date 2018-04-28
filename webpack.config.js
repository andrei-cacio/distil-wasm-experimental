const path = require('path');

module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "web"),
    filename: "index.js",
  },
  mode: "development"
};