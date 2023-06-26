use std::{fs, path::Path, process::{self, Stdio}, sync::{Arc, Mutex}};
use std::io::{Write, Read};
use threadpool::ThreadPool;
use serde::Deserialize;
use config::{Config, ConfigError};
use indicatif::ProgressBar;

#[derive(Debug, Deserialize)]
struct Settings {
    in_dir: String,
    out_dir: String,
    exec_command: String,
    judge_command: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(config::File::with_name("./settings")).build()?;

        s.try_deserialize()
    }
}

fn execute(in_file_path: &Path, out_file_path: &Path, exec_command:Arc<String>, judge_command: Arc<String>) -> u64{
    let exec_command = &*exec_command;
    let judge_command = &*judge_command;
    // ==実行部分==
    let mut in_file = fs::File::open(in_file_path).expect("Error: Faild to open the input file");
    let mut buf = Vec::new();
    in_file.read_to_end(&mut buf).expect("Error: Faild to read the input file");

    let ps = process::Command::new(exec_command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("exec failed");
            eprintln!("{e}");
            std::process::exit(1)
        });

    // 入力を渡す
    let mut stdin = ps.stdin.as_ref().unwrap();
    stdin.write_all(&buf).unwrap();

    // 出力を書き込む
    let output = ps.wait_with_output().unwrap();
    let mut file = fs::File::create(out_file_path).unwrap();
    file.write_all(&output.stdout).unwrap();


    // ==採点部分==
    let ps = process::Command::new(judge_command)
        .args([&in_file_path, &out_file_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("judge failed");
            eprintln!("{e}");
            std::process::exit(1)
        });

    // スコアが　Score = 1234 と出力されているとする
    let output = ps.wait_with_output().unwrap();
    let out = String::from_utf8(output.stdout).unwrap();
    let spl: Vec<&str> = out.split("=").collect();
    let score: u64 = spl[1].trim().parse().unwrap();

    score
}

fn main() {
    let settings = Settings::new().unwrap();
    let in_file_path = Path::new(&settings.in_dir);
    let out_file_path = Path::new(&settings.out_dir);

    if out_file_path.is_dir() {
        fs::remove_dir_all(out_file_path).unwrap();
    }
    fs::create_dir(&out_file_path).unwrap();

    let pool = ThreadPool::new(4);

    // 並列実行
    let total_score = Arc::new(Mutex::new(0));
    let min_score = Arc::new(Mutex::new((std::u64::MAX, "".to_string())));
    let entries = fs::read_dir(&in_file_path).unwrap();
    let exec_command = Arc::new(settings.exec_command);
    let judge_command = Arc::new(settings.judge_command);
    
    // プログレスバー
    let n = fs::read_dir(&in_file_path).unwrap().count() as u64;
    let bar = ProgressBar::new(n);
    let bar  = Arc::new(Mutex::new(bar));

    for entry in entries {
        let in_file_path = entry.unwrap().path();
        let out_file_path = out_file_path.join(in_file_path.file_name().unwrap());

        let total_score_clone = Arc::clone(&total_score);
        let min_score_clone = Arc::clone(&min_score);

        let exec_command_clone = Arc::clone(&exec_command);
        let judge_command_clone = Arc::clone(&judge_command);
        let bar_clone = Arc::clone(&bar);
        
        pool.execute( move || {
            let value = execute(&in_file_path, &out_file_path, exec_command_clone, judge_command_clone );
            let file_name = in_file_path.file_name().unwrap().to_str().unwrap().to_string();

            // スコアの集計
            let mut num = total_score_clone.lock().unwrap();
            *num += value;
            let mut min = min_score_clone.lock().unwrap();
            if min.0 > value  {
                min.0 = value;
                min.1 = file_name.clone();
            }           
            // バーを進める
            bar_clone.lock().unwrap().inc(1);
        });
    }

    pool.join();
    println!("total: {}", *total_score.lock().unwrap());
    let pair = min_score.lock().unwrap();
    println!("min: {}, file: {}", pair.0, pair.1);
}
