const path = require('node:path')
const CopyWebpackPlugin = require('copy-webpack-plugin')

module.exports = {
  entry: {
    'background/background': './src/background/background.ts',
    'content-scripts/dmm-extractor': './src/content-scripts/dmm-extractor.ts',
    'content-scripts/dlsite-extractor': './src/content-scripts/dlsite-extractor.ts',
    'popup/popup': './src/popup/popup.ts',
  },

  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].js',
    clean: true,
  },

  module: {
    rules: [
      {
        test: /\.ts$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },

  resolve: {
    extensions: ['.ts', '.js'],
  },

  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        { from: 'manifest.json', to: 'manifest.json' },
        { from: 'src/popup/popup.html', to: 'popup/popup.html' },
        { from: 'src/popup/styles.css', to: 'popup/styles.css' },
        { from: 'src/config/*.json', to: 'config/[name][ext]' },
      ],
    }),
  ],

  mode: 'development',
}
