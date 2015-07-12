require('imports?this=>window!modernizr');
require('fastclick');
var foundation = require('imports?jQuery=jquery!foundation-sites');
require('imports?jQuery=jquery!foundation.topbar');

module.exports = {
  "foundation": foundation,
  "react": require('react'),
  "$": require("jquery")
};
