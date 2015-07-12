require(['style/index']);
require(['script/vendor'], function(vendor) {
  var React = vendor.react;
  var $ =  vendor.$;

  var DEFAULTS = {
    workMin: 25,
    breakMin: 5,
    longBreakMin: 15,

    // stage constants
    stgWork: 1,
    stgBreak: 2,
    stgLongBreak: 3
  };

  //option components
  var OptCmp = React.createClass({
    render: function() {
      return (
        <div>
          opt
        </div>
      );
    }

  });

  //main components
  var MainCmp = React.createClass({
    render: function() {
      return (
        <div>
          main cmp
        </div>
      );
    }

  });

  //mainApp
  var MainApp = React.createClass({
    getDefaultProps: function() {
      return {
         mainName:"Pomodoro",
         mainURL:"http://pomodorotechnique.com",
         optName: "Option"
      };
    },

    showOpt: function() {
      this.setState(function(pSt) {
        return {showOpt: !pSt.showOpt};
      });
    },

    getInitialState: function() {
      return {showOpt: false};
    },

    render: function() {
      var showOpt = this.state.showOpt;
      return (
        <div>
          <div className="row">
            <div className="small-12">
              <nav id="stb" className="top-bar" data-topbar role="navigation">
                <ul className="title-area">
                  <li className="name">
                    <h1><a href={this.props.mainURL} target="_blank">{this.props.mainName}</a></h1>
                  </li>
                  <li className="toggle-topbar menu-icon"><a href="javascript:void(0);"><span>Menu</span></a></li>
                </ul>

                <section className="top-bar-section">
                    <ul className="right">
                      <li><a href="javascript:void(0);" onClick={this.showOpt}>{this.props.optName}</a></li>
                    </ul>
                </section>
              </nav>
            </div>
          </div>

          <div className="row">
            <div>
              <div className={showOpt ? "small-8 columns" : "small-12"}>
                <MainCmp />
              </div>

              <div className={showOpt ? "small-4 columns" : "hide"}> 
                <OptCmp />
              </div>
            </div>
          </div>
        </div>
      );
    }
  });

  React.render(<MainApp />, document.body, function() {
    $("#stb").foundation();
    $("#main").foundation();
  });

});
