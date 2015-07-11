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
    test: "./script/test.js"
  },

  output: {
    path: getDir('./build'),
    filename: "[name].js"
  },
  module: {
    loaders: [
      { test: /\.css$/, loader: "style!css" },
      { test: /\.png$/, loader: "url?limit=8192" },
      { test: /\.scss$/, loader: "style!css!sass?outputStyle=expanded"}
    ]
  },

  plugins: [
    new webpack.ResolverPlugin([
      new webpack.ResolverPlugin.DirectoryDescriptionFilePlugin("bower.json", ["main"])
    ])

  ],

  resolve: {
    root: [getDir('bower_components')],
    extensions: ["", ".js", ".scss"],
    modulesDirectories: ['node_modules', 'src']
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
