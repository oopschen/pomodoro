#[macro_use]
extern crate clap;
extern crate ncurses;

mod pomodoro;

fn main() {
    use clap::App;

    let cmd_yaml = load_yaml!("cmd.yml");
    let cmd_matches = App::from_yaml(cmd_yaml).get_matches();
    println!("break-time={}, long-break-time={}, work-time={}, threadhold={}",
             cmd_matches.value_of("break").unwrap(),
             cmd_matches.value_of("longbreak").unwrap(),
             cmd_matches.value_of("work").unwrap(),
             cmd_matches.value_of("longbreakthreadhold").unwrap()
             );
}
