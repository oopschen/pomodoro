'use strict';
var path = require('path');
var _ = require('underscore');

module.exports = function(grunt) {
  // jshint-opts
  var jshintOpts = _.defaults({node:true}, require(path.join(__dirname, "jshintrc.js")));

  // Project configuration.
  grunt.initConfig({
    // Metadata.
    jshint: {
      options: jshintOpts,
      src: ['Gruntfile.js', 'webpack.config.js']
    },

    clean: {
      build: {
        src: ["./build/**/*"]
      }
    },

    env: {
      dev: {
        NODE_ENV: "development"
      },

      prod: {
        NODE_ENV: "production"
      }
    },
    
    'gh-pages': {
        options: {
          base: 'build'
        },
        src: ['**']
    }
  });

  // These plugins provide necessary tasks.
  grunt.loadNpmTasks('grunt-contrib-jshint');
  grunt.loadNpmTasks('grunt-express-server');
  grunt.loadNpmTasks('grunt-webpack');
  grunt.loadNpmTasks('grunt-contrib-clean');
  grunt.loadNpmTasks('grunt-env');
  grunt.loadNpmTasks('grunt-gh-pages');

  var baseTask = ['clean:build', 'jshint'],
      taskProd = ['env:prod', 'setup', 'webpack:serv'];

  grunt.registerTask('default', baseTask.concat(taskDev));
  grunt.registerTask('prod', baseTask.concat(taskProd));

};
