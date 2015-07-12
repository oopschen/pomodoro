'use strict';
var webpack = require('webpack');
var path = require('path');
var _ = require('underscore');

module.exports = function(grunt) {
  var webpackConfig = require(path.join(__dirname, "webpack.config.js"));
  var isDebug = "prod" !== process.env["m"];
  var wCfg = !isDebug ? _.defaults({watch:false}, webpackConfig) : webpackConfig;
  if (!isDebug && wCfg.plugins) {
    wCfg.plugins.push(new webpack.optimize.UglifyJsPlugin({
      mangle: {
        except: ['$super', '$', 'exports', 'require']
      },
      compress: {
        warnings: false
      }
    }));
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
