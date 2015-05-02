# Pomodoro tools  
This is a easy use, command line based tool for the good of pomodoro.  
  
## Feature  
* timer for work  
* timer for break
* cusimized notify sound
  
## Usage  
  
    pomodoro [-h]  [-w 25] [-r 5] [-n /home/ray/notify.sound] [-l 15] [-c 4]
      
  
## Prerequest  
* mpv installed  
* go >=1.5

## Keyboard  
<table>  
  <tr>  
    <td>Key</td>
    <td>Illustration</td>
  </tr>  
  
  <tr>  
    <td>r</td>
    <td>rest all timer and go to ready state for work time</td>
  </tr>  

  <tr>  
    <td>n</td>
    <td>go to next step</td>
  </tr>  

  <tr>  
    <td>k</td>
    <td>stop</td>
  </tr>  
</table>
  
## Installation  
  
    export GOPATH=$(pwd)  
    go build -o [dir]/pomodoro oopschen.github.com/org/ray/
    cp notify.mp3 [dir]
