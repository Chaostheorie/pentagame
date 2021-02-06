const path = require('path');
const dist = path.resolve(__dirname, './dist/');
const webpack = require('webpack');

/*
SASS/ PurgeCSS is not handled here, because there have been some problems with setting it up consistently
If you know webpack and wanna give those two a try feel free to open a pull request with the required changes
for reference see: /scripts/compile.sh
*/

let banner = new webpack.BannerPlugin({
    banner: `
    Pentagame Online Static Asset ([name]) - Under GPL v3.0 @ Cobalt <chaosthe0rie@pm.me>
    The pentagame brand is under owned by Jan Suchanek and the board design is available under CC BY-SA-NC
     chunkhash:[chunkhash], filebase:[base], query:[query], file:[file]
    `,
});

module.exports = {
    mode: 'development',
    devtool: 'source-map', // don't ever use inline-source-map
    module: {
        rules: [
            {
                test: /\.ts?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
        ],
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    entry: {
        utils: './ts/utils.ts',
        main: './ts/main.ts',
        game: './ts/game.ts',
        settings: './ts/settings.ts',
        overview: './ts/overview.ts',
        main: './ts/main.ts',
    },
    plugins: [banner],
    output: {
        path: dist,
        publicPath: '/static/ts/',
        filename: '[name].js',
    },
};
