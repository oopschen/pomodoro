var $ = require('jquery');
var defs = {
  workMin: 25,
  breakMin: 5,
  longBreakMin: 15,
  lbMinThrd: 4
};

var ST_UNDEF,
  ST_WORK = 1,
  ST_BREAK = 2,
  ST_LONG_BREAK = 3,
  ST_CNT_DOWN = 4;

// constructor for pomo behaviar
// @param callback callback when timeout callback(flag, data)
// @param options option for pomodoro {}
var pomo = function(callback, options) {
  this._cb = callback;
  this._st = ST_UNDEF;
  this._wCnt = 0;
  this._interHdl = undefined;
  this.update(options);
};

// defaults for props
pomo.DEFAULTS = defs;

// callback flag
pomo.FLAG = {
  "END_WK": ST_WORK, // work
  "END_BK": ST_BREAK,  // break
  "END_L_BK": ST_LONG_BREAK, //long break
  "CNT_DOWN": ST_CNT_DOWN // count down
};

// do the next job based on the logic
// if stage == undfined:
//  start work
// else if st == work
//  if cnt reach threadhold
//    start long break
//  else 
//    start break
// else 
//    set st 2 undefined
pomo.prototype.next = function() {
  this._clear();
  if(ST_WORK === this._st) {
    if(this._state.lbMinThrd > this._wCnt) {
      this._start(1, this._state.breakMin * 60, ST_BREAK);

    } else {
      this._start(1, this._state.longBreakMin * 60, ST_LONG_BREAK);

    }

  } else {
    this._wCnt ++;
    this._start(1, this._state.workMin * 60, ST_WORK);

  }
};

// reset pomodoro as after the init
pomo.prototype.reset = function() {
  this._st = ST_UNDEF;
  this._wCnt = 0;
  this._clear();
};

// clear internal
pomo.prototype._clear = function() {
  if(undefined !== this._interHdl) {
    clearInterval(this._interHdl);
    this._interHdl = undefined;
  }
};

// start timer
pomo.prototype._start = function(sec, cnt, flag) {
  var tmpCnt = 0;

  this._st = flag;
  this._interHdl = setInterval(function() {
    if(tmpCnt++ < cnt) {
      this._cb(ST_CNT_DOWN, sec * (cnt - tmpCnt));

    } else {
      this._clear();
      this._cb(flag);

    }
  }.bind(this), sec * 1000);
};

pomo.prototype.getSetting = function() {
  return this._state;
};

pomo.prototype.update= function(options) {
  this._state = !options || !options.workMin ? defs : $.extend({}, defs, options);
};

module.exports = pomo;
