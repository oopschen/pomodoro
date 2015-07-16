require('imports?this=>window!modernizr');
require('fastclick');
var jq = require("jquery");
var foundation = require('imports?jQuery=jquery!foundation');
require('imports?jQuery=jquery!foundation.topbar');

module.exports = {
  "foundation": foundation,
  "react": require('react/addons'),
  "$": jq,
  "rhaboo": require("rhaboo"),
  "ps": require("pubsub-js")
};

