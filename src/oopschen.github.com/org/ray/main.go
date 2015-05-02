// Package main provides ...
package main

import (
	"flag"
	"fmt"
	"os/exec"
	"strings"
	"time"
)

type Option struct {
	workMin          uint
	breakMin         uint
	longBreakWorkCnt uint
	longBreakMin     uint
	notifySoundPath  string
}

func main() {
	option := parseArgs()
	if nil == option {
		return
	}

	doJob(option)
}

func parseArgs() *Option {
	// -w work time min -r break time min -n notify sound path
	var (
		option                                                        *Option = &Option{}
		showHelp                                                      bool
		defWorkMin, defBreakMin, defLongBreakWorkCnt, defLongBreakMin uint   = 25, 5, 4, 15
		defNotifySndPath                                              string = "./notify.mp3"
	)

	flag.UintVar(&option.workMin, "w", defWorkMin, "work time minutes")
	flag.UintVar(&option.breakMin, "r", defBreakMin, "break time minutes")
	flag.UintVar(&option.longBreakWorkCnt, "c", defLongBreakWorkCnt, "after which work time, take the long break ")
	flag.UintVar(&option.longBreakMin, "l", defLongBreakMin, "long break minutes")
	flag.StringVar(&option.notifySoundPath, "n", defNotifySndPath, "notify sound path, only support mp3")
	flag.BoolVar(&showHelp, "h", false, "break time minutes")

	flag.Parse()

	if showHelp {
		fmt.Printf("Usgae:\n")
		fmt.Printf("\t-w work time minutes, default %dmins\n", defWorkMin)
		fmt.Printf("\t-r break time minutes, default %dmins\n", defBreakMin)
		fmt.Printf("\t-n notify sound path, default %s\n", defNotifySndPath)
		fmt.Printf("\t-h show this help\n")
		return nil
	}
	return option
}

func doJob(option *Option) {
	// -w
	const (
		MODE_WORK       = 1 << iota
		MODE_BREAK      = 1 << iota
		MODE_LONG_BREAK = 1 << iota
	)
	var (
		isStop           bool = false
		mode                  = MODE_WORK
		sleepTime             = option.workMin
		cmd              string
		workCnt          uint = 0
		longBreakWorkCnt      = option.longBreakWorkCnt - 1
		playSndCmd            = exec.Command("mpv", option.notifySoundPath)
	)

	for !isStop {
		// check long break
		if longBreakWorkCnt < workCnt {
			sleepTime = option.longBreakMin
			mode = MODE_LONG_BREAK
			workCnt = 0
			fmt.Printf("Start %dmins long break!!!\n", sleepTime)
		} else if MODE_WORK == mode {
			sleepTime = option.workMin
			fmt.Printf("Start %dmins work!!!\n", sleepTime)
		} else {
			sleepTime = option.breakMin
			fmt.Printf("Start %dmins break!!!\n", sleepTime)
		}

		select {
		case <-time.After(time.Duration(sleepTime) * time.Minute):
			if MODE_BREAK == mode || MODE_LONG_BREAK == mode {
				mode = MODE_WORK
			} else if MODE_WORK == mode {
				workCnt++
				mode = MODE_BREAK
			}

			// play sound
			playSndCmd.Run()
			fmt.Printf("Timeout!!!\nWait for your command(r|k|n)\n")

		}

		// scan command
		for isB := false; !isB; {
			scanNum, err := fmt.Scanf("%1s\n", &cmd)
			if 1 > scanNum || nil != err {
				fmt.Printf("Valid command is r(reset timer), n(go to next step), k(stop this program)\n")
				continue
			}

			switch strings.ToLower(cmd) {
			case "r":
				mode = MODE_WORK
				workCnt = 0
				isB = true
				break
			case "n":
				isB = true
				break
			case "k":
				isB = true
				isStop = true
				break
			}

		}

	}
}
