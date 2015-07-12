require(['style/index']);
require(['script/vendor'], function(vendor) {
  var React = vendor.react;
  var $ =  vendor.$;

  var DEFAULTS = {
    workMin: 25,
    breakMin: 5,
    longBreakMin: 15,
    lbMinThrd: 4
  };

  var CONST_STAGE = {
    // stage constants
    stgWork: 1,
    stgBreak: 2,
    stgLongBreak: 3
  };

  var CONST_STORE_KEY = "pormodoro_opt";

  var optStore = vendor.rhaboo.persistent("pomodoroOptStore");

  //option components
  var OptCmp = React.createClass({
    mixins: [React.addons.LinkedStateMixin],
    render: function() {
      return (
        <form>
          <div className="row">
            <label>{this.props.workTimeName}
              <input type="text" valueLink={this.linkState('workMin')} />
            </label>
          </div>

          <div className="row">
            <label>{this.props.breakTimeName}
              <input type="text" valueLink={this.linkState('breakMin')} />
            </label>
          </div>

          <div className="row">
            <label>{this.props.longbreakTimeName}
              <input type="text" valueLink={this.linkState('longBreakMin')} />
            </label>
          </div>

          <div className="row">
            <label>{this.props.longBreakThreadhold}
              <input type="text" valueLink={this.linkState('lbMinThrd')} />
            </label>
          </div>

          <div className="row">
            <div className="small-2 small-offset-3 columns">
              <a href="javascript:void(0);" className="button round" onClick={this.saveData}>{this.props.submitBtnName}</a>
            </div>
          </div>

        </form>
      );
    },

    getInitialState: function() {
      var val = optStore[CONST_STORE_KEY];
      return !val ? $.extend({}, DEFAULTS) : val;
    },

    getDefaultProps: function() {
      return {
        workTimeName: 'Work Time(Min)',
        breakTimeName: 'Break Time(Min)',
        longbreakTimeName: 'Long Break Time(Min)',
        longBreakThreadhold: 'Long Break ThreadHold(Time)',
        submitBtnName: 'Save'
      };
    },

    saveData: function() {
      optStore.write(CONST_STORE_KEY, this.state);
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
         optName: "Option",
         closeOptname: "Close"
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
                      <li><a href="javascript:void(0);" onClick={this.showOpt}>{showOpt? this.props.closeOptname : this.props.optName}</a></li>
                    </ul>
                </section>
              </nav>
            </div>
          </div>

          <div className="row">
            <div>
              <div className={showOpt ? "small-9 columns" : "small-12"}>
                <MainCmp />
              </div>

              <div className={showOpt ? "small-3 columns" : "hide"}> 
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