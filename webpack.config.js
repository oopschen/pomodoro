var webpack = require('webpack');
var path = require('path');

var getDir = function() {
  var args = Array.prototype.slice.call(arguments);
  return path.join.apply(path, [__dirname].concat(args));
};

module.exports = {
  // webpack options 
  context: getDir('./src'),

  entry: {
    index: "./script/index.js",
    style: "./style/index.scss"
  },

  output: {
    path: getDir('./build'),
    filename: "[hash].js"
  },

  module: {
    loaders: [
      { test: /\.css$/, loader: "style!css" },
      { test: /\.(jpeg|png|jpg)$/, loader: "url?limit=512" },
      { 
        test: /\.scss$/,
        loader: "style!css!sass?outputStyle=expanded&" +
          "includePaths[]=" + getDir('node_modules', 'foundation-sites', 'scss')
      }
    ]
  },

  plugins: [
    new webpack.optimize.CommonsChunkPlugin({name: "index", filename: "[hash].chk.js"})
  ],

  resolve: {
    root: [getDir("src"), getDir(".")]
  },

  progress: false, // Don't show progress 
  // Defaults to true 

  failOnError: true, // don't report error to grunt if webpack find errors 
  // Use this if webpack errors are tolerable and grunt should continue 

  watch: true, // use webpacks watcher 
  // You need to keep the grunt process alive 

  keepalive: true, // don't finish the grunt task 
  // Use this in combination with the watch option 

  devtool: 'eval'
};
