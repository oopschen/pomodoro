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

    express: {
      run: {
        options: {
          script: __dirname + '/server.js',
          port: 5000,
          delay:1000
        }
      }
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
    }

  });

  // These plugins provide necessary tasks.
  grunt.loadNpmTasks('grunt-contrib-jshint');
  grunt.loadNpmTasks('grunt-express-server');
  grunt.loadNpmTasks('grunt-webpack');
  grunt.loadNpmTasks('grunt-contrib-clean');
  grunt.loadNpmTasks('grunt-env');

  var baseTask = ['clean:build', 'jshint'],
      taskDev = ['env:dev', 'setup', 'express:run', 'webpack:serv'],
      taskProd = ['env:prod', 'setup', 'webpack:serv'];

  // Default task.
  grunt.registerTask('setup', function() {
    grunt.config('webpack', {
      serv: require(path.join(__dirname, "webpack.config.js"))
    });
  });

  grunt.registerTask('default', baseTask.concat(taskDev));
  grunt.registerTask('prod', baseTask.concat(taskProd));

};
