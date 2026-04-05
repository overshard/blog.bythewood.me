const path = require("path");

const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const CssMinimizerPlugin = require("css-minimizer-webpack-plugin");
const TerserPlugin = require("terser-webpack-plugin");
const BASE_DIR = __dirname;

module.exports = {
  entry: {
    "admin.global": "./admin/static_src/global/index.js",
    "admin.editor": "./admin/static_src/editor/index.js",
    base: "./blog/static_src/index.js",
    pages: "./pages/static_src/index.js",
  },
  output: {
    path: path.resolve(BASE_DIR, "blog/static"),
    filename: "[name].js",
  },
  optimization: {
    minimize: true,
    minimizer: [
      new CssMinimizerPlugin(),
      new TerserPlugin(),
    ],
  },
  plugins: [
    new MiniCssExtractPlugin({
      filename: "[name].css",
    }),
  ],
  module: {
    rules: [
      // Extract all CSS into their own files
      {
        test: /\.css$/,
        use: [MiniCssExtractPlugin.loader, "css-loader"],
      },
      {
        test: /\.scss$/,
        use: [MiniCssExtractPlugin.loader, "css-loader", "sass-loader"],
      },
      // Copy all images to the build directory
      {
        test: /\.(png|jpg|gif|svg|webp)$/,
        use: [
          {
            loader: "file-loader",
            options: {
              name: "[name].[ext]",
              outputPath: "images/",
            },
          },
        ],
      },
    ],
  },
};
