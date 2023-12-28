use clap::Parser;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::sleep;
use std::sync::Mutex;
use std::time::Duration;
use std::thread;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    new_engine: PathBuf,
    old_engine: PathBuf,
}

enum GameResult {
    Error,
    NewLoss,
    NewWin,
    Draw,
    Stalemate,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = run_game(&cli.new_engine, &cli.old_engine);
    match result {
        GameResult::Draw => {
            println!("It was a draw");
            ExitCode::SUCCESS
        }
        GameResult::Error => {
            println!("It was a error");
            ExitCode::FAILURE
        }
        GameResult::NewLoss => {
            println!("The New engine lost");
            ExitCode::SUCCESS
        }
        GameResult::NewWin => {
            println!("The new engine won");
            ExitCode::FAILURE
        }
        GameResult::Stalemate => {
            println!("Stalemate Detected");
            ExitCode::SUCCESS
        }
    }
}

fn run_game(new_engine: &PathBuf, old_engine: &PathBuf) -> GameResult {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    start_engine(tx1, rx2, new_engine).unwrap();

    tx2.send(String::from("Command 1\n")).unwrap();
    start_command_thread(Mutex::new(tx2));

    for line in rx1 {
        println!("Got this back: {}", line);
    }
    GameResult::Error
}

fn start_engine(
    sender: Sender<String>,
    receiver: Receiver<String>,
    engine_path: &PathBuf,
) -> Result<i32, std::io::Error> {
    let child = Command::new(engine_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    thread::spawn(move || {
        let mut f = BufReader::new(child.stdout.unwrap());
        let mut stdin = child.stdin.unwrap();
        for line in receiver {
            stdin.write_all(line.as_bytes()).unwrap();
            let mut buf = String::new();
            match f.read_line(&mut buf) {
                Ok(_) => {
                    sender.send(buf).unwrap();
                    continue;
                }
                Err(e) => {
                    println!("an error!: {:?}", e);
                    break;
                }
            }
        }
    });
    return Ok(3);
}

fn start_command_thread(mutex: Mutex<Sender<String>>) {
    thread::spawn(move || {
        let sender = mutex.lock().unwrap();
        sleep(Duration::from_secs(3));
        sender
            .send(String::from("Command from the thread\n"))
            .unwrap();
    });
}