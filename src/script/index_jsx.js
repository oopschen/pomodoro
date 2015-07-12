require(['style/index']);
require(['script/vendor'], function(vendor) {
  var React = vendor.react;
  var container = document.getElementById("container");
  React.render(<h1>helloWorld</h1>, container);

});
