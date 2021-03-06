require('style/index');
require('imports?this=>window!modernizr');
require('fastclick');
var $ = require('jquery');
require('imports?jQuery=jquery!foundation');
require('imports?jQuery=jquery!foundation.topbar');
var Push = require('push.js');

require(['script/pomodoro'], function(Pomo) {
  var React = require('react'),
      ps = require('pubsub-js');

  var CONST_STORE_KEY = "pormodoro.opt",
      CONST_SAVE_OPT = "opt.save",
      CONST_UP_TIME = "timer.up";

  var optStore = require("rhaboo").persistent("pomodoro.optstore"),
      notifySnd = require('../snd/notify.mp3');
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
            <div className="small-6 small-centered columns">
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
        tipsTitle: "Keyboard shortcut",
        tipsContent: "Spacebar for next, r for next"
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

        var tPrompt;
        if (Pomo.FLAG.END_WK === data.flag) {
          tPrompt = "Work Done, take a break!";

        } else if (Pomo.FLAG.END_BK === data.flag || Pomo.FLAG.END_L_BK === data.flag) {
          tPrompt = "Break Time Over!";

        }
        this.setState({min:paddingZero(0), sec: paddingZero(0), timeoutPrompt: tPrompt, tt:true});
        Push.create(tPrompt, {
          timeout: 20000,
          icon: {
            x16: require('../img/end_x16.png'),
            x32: require('../img/end_x32.png')
          },
          body: 'Pomodoro Notification'
        });

        // play sound
        if (!player) {
          require(['jplayer'], function() {
            player = $('#player').jPlayer({
              swfPath: require('../../node_modules/jplayer/dist/jplayer/jquery.jplayer.swf'),
              preload: 'auto',
              cssSelectorAncestor: "#playerC",
              volume: 1,
              errorAlerts: false,
              warningAlerts: false,
              ready: function () {
                $(this).jPlayer("setMedia", {
                  mp3: notifySnd // Defines the mp3 url
                });

                player.jPlayer("play", 0);
              }
            });


          });

        } else {
            player.jPlayer("play", 0);

        }

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
        <div>
          <div className="row">
            <div className="small-12 column show-for-large-only">
              <div className="panel callout radius">
                <h6>{this.props.tipsTitle}</h6>
                <p>{this.props.tipsContent}</p>
              </div>
            </div>
          </div>

          <div className={this.state.tt ? "row" : "hide"}>
            <div className="small-8 small-centered column">
              <div data-alert className="alert-box success radius">
                {this.state.timeoutPrompt}
              </div>
            </div>
          </div>

          <div className="row">
            <div className="small-12 small-centered columns">

              <div className="row font-time">
                <div className="small-5 columns text-center">
                  {this.state.min}
                </div>

                <div className="small-2 columns text-center">
                  :
                </div>
                
                <div className="small-5 column text-center">
                  {this.state.sec}
                </div>
              </div>

            </div>
          </div>

          <div className="row">
            <div className="small-8 small-centered columns">
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
         closeOptname: "Close"
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
            <div className={showOpt ? "hide" : "small-12 columns"}>
              <MainCmp />
            </div>

            <div className={showOpt ? "small-12 columns" : "hide"}> 
              <OptCmp />
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

  });

});
