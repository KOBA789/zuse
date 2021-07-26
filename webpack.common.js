/*eslint-env node*/

const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");

module.exports = {
  output: {
    publicPath: '',
  },
  entry: {
    app: ["./js/index.ts", "./js/style.css"],
  },
  plugins: [
    new CleanWebpackPlugin(),
    new HtmlWebpackPlugin({
      title: "Zuse",
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "./rs/"),
      outDir: path.resolve(__dirname, "./rs/pkg"),
    }),
    new MiniCssExtractPlugin(),
  ],
  module: {
    rules: [
      {
        test: /\.[jt]sx?$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader",
          options: {
            cacheDirectory: true,
          },
        },
      },
      {
        test: /\.(eot|ttf|woff|woff2|svg|png|gif|jpe?g)$/,
        loader: "file-loader",
        options: {
          name: "[name].[ext]?[hash]",
          outputPath: "assets/",
        },
      },
    ],
  },
  resolve: {
    extensions: [".tsx", ".ts", ".js"],
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
