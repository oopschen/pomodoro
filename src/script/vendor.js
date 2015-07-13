require('imports?this=>window!modernizr');
require('fastclick');
var foundation = require('imports?jQuery=jquery!foundation-sites');
require('imports?jQuery=jquery!foundation.topbar');

module.exports = {
  "foundation": foundation,
  "react": require('react/addons'),
  "$": require("jquery"),
  "rhaboo": require("rhaboo/src/rocks/arr"),
  "ps": require("pubsub-js")
};

require('jplayer');
