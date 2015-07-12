require(['style/index']);
require(['script/vendor'], function(vendor) {
  var React = vendor.react;
  var $ =  vendor.$;
  var container = document.getElementById("container");

  // top bar components
  var SingleTopBar = React.createClass({
    render: function() {
      return (
        <nav className="top-bar" data-topbar role="navigation">
          <ul className="title-area">
            <li className="name">
              <h1><a href={this.props.mainURL} target="_blank">{this.props.mainName}</a></h1>
            </li>
            <li className="toggle-topbar menu-icon"><a href="javascript:void(0);"><span>Menu</span></a></li>
          </ul>

          <section className="top-bar-section">
              <ul className="right">
                <li><a href="javascript:void(0);" onClick={this.handleOpt}>{this.props.optName}</a></li>
              </ul>
          </section>
        </nav>
      );
    },

    handleOpt: function(evt) {
      console.log("opt click", evt);
    }

  });

  var singleTopBarProps = {
     mainName:"Pomodoro",
     mainURL:"http://pomodorotechnique.com",
     optName: "Option"
  };

  React.render(<SingleTopBar {...singleTopBarProps} />, container);

  
  $(container).foundation();

});
