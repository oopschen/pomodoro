var webpack = require('webpack');
var path = require('path');
var _ = require('underscore');

var getDir = function() {
  var args = Array.prototype.slice.call(arguments);
  return path.join.apply(path, [__dirname].concat(args));
};

module.exports = {
  // webpack options 
  context: getDir('./src'),

  entry: {
    index: ["./script/index_jsx.js"]
  },

  output: {
    path: getDir('./build/script'),
    filename: "[name].js"
  },

  module: {
    loaders: [
      { test: /\.css$/, loader: "style!css" },
      { test: /\.(jpeg|png|jpg)$/, loader: "url?limit=512" },
      { test: /_jsx\.js/, loader: "jsx" },
      { 
        test: /\.scss$/,
        loader: "style!css!sass?outputStyle=expanded&" +
          "includePaths[]=" + getDir('node_modules', 'foundation-sites', 'scss')
      }
    ],

    postLoaders: [
      {
        test: /\.js$/, // include .js files
        exclude: /node_modules/, // exclude any and all files in the node_modules folder
        loader: "jshint-loader"
      }
    ]
  },

  jshint: _.defaults(
    {
      failOnHint:true,
      browser: true
    },
    require(path.join(__dirname, "jshintrc.js"))),

  plugins: [
    new webpack.optimize.OccurenceOrderPlugin(true)
  ],

  resolve: {
    root: [
      getDir("src"),
      getDir("."),
      getDir('node_modules', 'foundation-sites', 'js', 'foundation'),
      getDir('node_modules', 'foundation-sites', 'js', 'vendor')
    ],
    extensions: ["", ".js", ".scss"]
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
