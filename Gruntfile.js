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
        src: ["build/**/*"]
      }
    },

    'gh-pages': {
        options: {
          base: 'build'
        },
        src: '**/*'
    },

    copy: {
      html: {
        expand: true,
        cwd: 'src/html',
        src: '**',
        dest: 'build',
      },
    }

  });

  // These plugins provide necessary tasks.
  grunt.loadNpmTasks('grunt-contrib-jshint');
  grunt.loadNpmTasks('grunt-contrib-clean');
  grunt.loadNpmTasks('grunt-contrib-copy');
  grunt.loadNpmTasks('grunt-gh-pages');

  var baseTask = ['clean:build', 'jshint'];

  grunt.registerTask('default', baseTask, function() {
    grunt.log.writeln('webpack-dev-server --content-base src/html');
  });
  
  grunt.registerTask('prod-action', function() {
    grunt.log.writeln('NODE_ENV=production webpack-dev-server --content-base src/html');
  });

  grunt.registerTask('prod', baseTask.concat(['copy:html', 'prod-action']));
};
