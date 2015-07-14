require(['style/index']);
require(['script/vendor', 'script/pomodoro'], function(vendor, Pomo) {
  var React = vendor.react,
      $ =  vendor.$,
      ps = vendor.ps;

  var CONST_STORE_KEY = "pormodoro.opt",
      CONST_SAVE_OPT = "opt.save",
      CONST_UP_TIME = "timer.up";

  var optStore = vendor.rhaboo.persistent("pomodoro.optstore");
  var notifySnd = require('file?name=[hash].[ext]!../snd/notify.mp3');
  var player;

  //pomodoro instance
  var pomoCB = function(flag, sec) {
    if (Pomo.FLAG.CNT_DOWN === flag) {
      ps.publish(CONST_UP_TIME, {sec:sec, flag: flag});

    } else {
      ps.publish(CONST_UP_TIME, {flag: flag});

    }
  };
  var pomoIns = new Pomo(pomoCB, optStore[CONST_STORE_KEY]);

  // utils
  var paddingZero = function(val) {
    return 10 > val ? ("0" + val) : val;
  };

  //option components
  var OptCmp = React.createClass({
    mixins: [React.addons.LinkedStateMixin],
    render: function() {
      return (
        <form>
          <div className="row">
            <div className="small-6 columns">
              <label>{this.props.workTimeName}
                <input type="text" valueLink={this.linkState('workMin')} />
              </label>
            </div>

            <div className="small-6 columns">
              <label>{this.props.breakTimeName}
                <input type="text" valueLink={this.linkState('breakMin')} />
              </label>
            </div>
          </div>

          <div className="row">
            <div className="small-6 columns">
              <label>{this.props.longbreakTimeName}
                <input type="text" valueLink={this.linkState('longBreakMin')} />
              </label>
            </div>

            <div className="small-6 columns">
              <label>{this.props.longBreakThreadhold}
                <input type="text" valueLink={this.linkState('lbMinThrd')} />
              </label>
            </div>
          </div>

          <div className="row">
            <div className="small-4 small-centered columns">
              <a href="javascript:void(0);" className="button round columns" onClick={this.saveData}>{this.props.submitBtnName}</a>
            </div>
          </div>

        </form>
      );
    },

    getInitialState: function() {
      var val = pomoIns.getSetting();
      return !val || undefined === val.workMin ? $.extend({}, Pomo.DEFAULTS) : val;
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
      pomoIns.update(this.state);
      ps.publish(CONST_SAVE_OPT);
    }

  });

  //main components
  var MainCmp = React.createClass({
    getDefaultProps: function() {
      return {
        btnNextName: "Next",
        btnResetName: "Reset",
      };
    },

    getInitialState: function() {
      ps.subscribe(CONST_UP_TIME, function(msg, data) {
        if (msg !== CONST_UP_TIME) {
          return;
        }

        if (Pomo.FLAG.CNT_DOWN === data.flag) {
          var min = Math.floor(data.sec / 60),
              sec = data.sec - min * 60;
          this.setState({min:paddingZero(min), sec: paddingZero(sec)});
          return;

        }

        // play sound
        player.jPlayer("play", 0);

        var tPrompt;
        if (Pomo.FLAG.END_WK === data.flag) {
          tPrompt = "Work Done, take a break!";

        } else if (Pomo.FLAG.END_BK === data.flag || Pomo.FLAG.END_L_BK === data.flag) {
          tPrompt = "Break Time Over!";

        }
        this.setState({min:paddingZero(0), sec: paddingZero(0), timeoutPrompt: tPrompt, tt:true});

      }.bind(this));

      return {
        tt: false,
        min: "00",
        sec: "00",
        timeoutPrompt: ""
      };
    },

    render: function() {
      return (
        <div className="row">

          <div className={this.state.tt ? "row" : "hide"}>
            <div className="small-5 small-centered column">
              <div data-alert className="alert-box success radius">
                {this.state.timeoutPrompt}
              </div>
            </div>
          </div>

          <div className="row">
            <div className="small-8 small-centered columns">
                <span className="text-center margin-20 font-15">{this.state.min}</span>
                <span className="text-center margin-20 font-12">:</span>
                <span className="text-center margin-20 font-15">{this.state.sec}</span>
            </div>
          </div>

          <div className="row">
            <div className="small-5 small-centered columns">
              <ul className="button-group round even-2">
                <li><a href="javascript:void(0);" className="button" onClick={this.hdlNext}>{this.props.btnNextName}</a></li>
                <li><a href="javascript:void(0);" className="button" onClick={this.hdlReset}>{this.props.btnResetName}</a></li>
              </ul>
            </div>
          </div>

        </div>
      );
    },

    hdlNext: function() {
      this.setState({tt:false});
      pomoIns.next();
    },

    hdlReset: function() {
      this.setState({tt:false, min: "00", sec: "00"});
      pomoIns.reset();
    },

    componentDidMount: function() {
      $(document).keydown(this.hdlKeyPress);
    },

    componentWillUnmount: function() {
      $(document).off("keydown");
    },

    hdlKeyPress: function(evt) {
      // spacebar 32, r 82
      switch(evt.which) {
        case 32:
          this.hdlNext();
          break;

        case 82:
          this.hdlReset();
          break;
      }
    }

  });

  //mainApp
  var MainApp = React.createClass({
    getDefaultProps: function() {
      return {
         mainName:"Pomodoro",
         mainURL:"http://pomodorotechnique.com",
         optName: "Option",
         closeOptname: "Close",
         tipsTitle: "Keyboard shortcut",
         tipsContent: "Spacebar for next, r for next"
      };
    },

    showOpt: function() {
      this.setState(function(pSt) {
        return {showOpt: !pSt.showOpt};
      });
    },

    getInitialState: function() {
      ps.subscribe(CONST_SAVE_OPT, function() {
        this.showOpt();
      }.bind(this));
      return {showOpt: false};
    },

    render: function() {
      var showOpt = this.state.showOpt;
      return (
        <div>
          <div className="row">
            <div className="small-12 column" id="stb">
              <nav className="top-bar" data-topbar role="navigation">
                <ul className="title-area">
                  <li className="name">
                    <h1><a href={this.props.mainURL} target="_blank">{this.props.mainName}</a></h1>
                  </li>
                  <li className="toggle-topbar menu-icon"><a href="javascript:void(0);"><span></span></a></li>
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
            <div className="row">
              <div className="small-10 small-centered column">
                <div className="panel callout radius">
                  <h6>{this.props.tipsTitle}</h6>
                  <p>{this.props.tipsContent}</p>
                </div>
              </div>
            </div>

            <div className="row">
              <div className={showOpt ? "hide" : "small-12 columns"}>
                <MainCmp />
              </div>

              <div className={showOpt ? "small-12 columns" : "hide"}> 
                <OptCmp />
              </div>
            </div>

          </div>

          <div className="hide">
            <div id="player"></div>
            <div id="playerC"><a className="jp-play" href="javascript:void(0);">Play</a></div>
          </div>
        </div>
      );
    }

  });

  React.render(<MainApp />, document.body, function() {
    $("#stb").foundation();
    $("#main").foundation();

    player = $('#player').jPlayer({
      swfPath: require('file?name=[hash].[ext]!../../node_modules/jplayer/dist/jplayer/jquery.jplayer.swf'),
      preload: 'auto',
      cssSelectorAncestor: "#playerC",
      volume: 1,
      errorAlerts: false,
      warningAlerts: false,
      ready: function () {
        $(this).jPlayer("setMedia", {
          mp3: notifySnd // Defines the mp3 url
        });
      }
    });
  });

});
