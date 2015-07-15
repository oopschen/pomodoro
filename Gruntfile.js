'use strict';
var webpack = require('webpack');
var path = require('path');
var _ = require('underscore');

module.exports = function(grunt) {
  var webpackConfig = require(path.join(__dirname, "webpack.config.js"));
  var isDebug = "prod" !== process.env["m"];
  var wCfg = webpackConfig;
  if (!isDebug && wCfg.plugins) {
    if (wCfg.watch) {
      wCfg.watch = false;
    }

    if (wCfg.devtool) {
      delete wCfg.devtool;
    }

    var plugins = [
      new webpack.DefinePlugin({
        'process.env': {
          'NODE_ENV': '"production"'
        }
      }),

      new webpack.optimize.UglifyJsPlugin({
        mangle: true,
        compress: {
          warnings: false
        },
        output: {
          comments:false,
          space_colon: false
        }
      })
    ];

    wCfg.plugins = wCfg.plugins.concat(plugins);

  }

  if (isDebug && wCfg.jshint) {
    wCfg.jshint["devel"] = true;
  }

  // jshint-opts
  var jshintOpts = _.defaults({node:true}, require(path.join(__dirname, "jshintrc.js")));

  // Project configuration.
  grunt.initConfig({
    // Metadata.
    jshint: {
      options: jshintOpts,
      src: ['Gruntfile.js', 'webpack.config.js']
    },

    express: {
      run: {
        options: {
          script: __dirname + '/server.js',
          port: 5000,
          delay:1000
        }
      }
    },

    webpack: {
      serv: wCfg
    }

  });

  // These plugins provide necessary tasks.
  grunt.loadNpmTasks('grunt-contrib-jshint');
  grunt.loadNpmTasks('grunt-express-server');
  grunt.loadNpmTasks('grunt-webpack');

  // Default task.
  grunt.registerTask('default',
                     isDebug ? ['jshint', 'express:run', 'webpack:serv'] : ['jshint', 'webpack:serv']);

};
