const { merge } = require('webpack-merge');
const common = require('./webpack.config.js');
const TerserPlugin = require('terser-webpack-plugin');

/*
SASS/ PurgeCSS is not handled here, because there have been some problems with setting it up consistently
If you know webpack and wanna give those two a try feel free to open a pull request with the required changes
for refernce see: /scripts/compile.sh
*/

module.exports = merge(common, {
    mode: 'production',
    devtool: '', // reduces binary size significantly
    optimization: {
        minimize: true,
        minimizer: [
            new TerserPlugin({
                parallel: true,
            }),
        ],
        removeEmptyChunks: false,
        sideEffects: true,
        mergeDuplicateChunks: true,
        nodeEnv: 'production',
    },
});
