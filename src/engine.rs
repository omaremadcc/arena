use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

#[derive(Debug)]
pub enum AnalysisLine {
    Move(String),
    Depth {
        depth: Option<String>,
        selective_depth: Option<String>,
        score: Option<String>,
        best_move: Option<String>,
        nodes: Option<String>,
        time: Option<String>,
    },
}
impl AnalysisLine {
    fn new(line: String) -> Option<AnalysisLine> {
        let line = line.trim().replace("\n", "");
        let args = line.split_whitespace().collect::<Vec<_>>();
        if line.starts_with("bestmove") {
            return Some(AnalysisLine::Move(args[1].to_string()));
        } else if line.starts_with("info") {
            let mut depth = None;
            let mut nodes = None;
            let mut best_move = None;
            let mut time = None;
            let mut score = None;

            let depth_index = args.iter().position(|str| str == &"depth");
            if let Some(depth_index) = depth_index {
                if let Some(depth_str) = args.get(depth_index + 1) {
                    depth = Some(depth_str.to_string());
                }
            }
            let score_index = args.iter().position(|str| str == &"cp" || str == &"mate");
            if let Some(score_index) = score_index {
                if let Some(score_str) = args.get(score_index + 1) {
                    score = Some(score_str.to_string());
                }
            }
            let nodes_index = args.iter().position(|str| str == &"nodes");
            if let Some(nodes_index) = nodes_index {
                if let Some(nodes_str) = args.get(nodes_index + 1) {
                    nodes = Some(nodes_str.to_string());
                }
            }
            let best_move_index = args.iter().position(|str| str == &"pv");
            if let Some(best_move_index) = best_move_index {
                if let Some(best_move_str) = args.get(best_move_index + 1) {
                    best_move = Some(best_move_str.to_string());
                }
            }
            let time_index = args.iter().position(|str| str == &"time");
            if let Some(time_index) = time_index {
                if let Some(time_str) = args.get(time_index + 1) {
                    time = Some(time_str.to_string());
                }
            }

            return Some(AnalysisLine::Depth {
                depth: depth,
                selective_depth: None,
                score,
                best_move: best_move,
                nodes,
                time,
            });
        }
        None
    }
}

#[derive(Clone)]
pub enum EngineOption {
    CHECK {
        name: String,
        value: bool,
    },
    SPIN {
        name: String,
        value: i32,
        min: Option<i32>,
        max: Option<i32>,
    },
} //

pub struct Engine {
    pub path: String,
    pub name: String,
    pub engine_options: Vec<EngineOption>,
    pub engine_handle: Option<EngineHandle>,
    pub analysis: Vec<AnalysisLine>,
    pub is_show: bool,
} //

impl Clone for Engine {
    fn clone(&self) -> Self {
        Engine {
            path: self.path.clone(),
            name: self.name.clone(),
            engine_options: self.engine_options.clone(),
            engine_handle: None,
            analysis: Vec::new(),
            is_show: true,
        }
    }
}

impl Engine {
    pub fn new(path: &str, name: &str) -> Self {
        let path = Path::new(path);

        if !path.exists() {
            panic!("Engine path does not exist");
        } else if !path.is_file() {
            panic!("Engine path is not a file");
        }
        if let Some(extension) = path.extension() {
            if extension != "exe" && extension != "" {
                panic!("Engine file is not an executable");
            }
        } else {
            panic!("Engine file has no extension");
        }

        let mut engine_process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start engine process");

        let mut stdin = engine_process
            .stdin
            .take()
            .expect("Failed to take engine stdin");
        let mut stdout = BufReader::new(
            engine_process
                .stdout
                .take()
                .expect("Failed to take engine stdout"),
        );

        stdin
            .write_all("uci\n".as_bytes())
            .expect("Failed to write 'uci' to engine stdin");

        let is_uci_ok;
        loop {
            let mut line = String::new();
            let _ = stdout.read_line(&mut line);
            if line.starts_with("uciok") {
                is_uci_ok = true;
                break;
            }
        }
        if !is_uci_ok {
            panic!("Engine is not UCI compatible");
        }

        stdin
            .write_all("quit\n".as_bytes())
            .expect("Error stopping connection");

        let mut engine = Engine {
            path: path.to_str().unwrap().to_string(),
            name: name.to_string(),
            engine_options: Vec::new(),
            engine_handle: None,
            analysis: Vec::new(),
            is_show: true,
        };
        engine.engine_options = engine.detect_engine_options();

        engine
    } //

    pub fn spawn_handle(&mut self) {
        let (cmd_tx, cmd_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (evt_tx, evt_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

        let mut child_process = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start engine process");
        let mut stdin = child_process
            .stdin
            .take()
            .expect("Failed to take engine stdin");
        let stdout = BufReader::new(
            child_process
                .stdout
                .take()
                .expect("Failed to take engine stdout"),
        );

        // stdin writer task
        thread::spawn(move || {
            while let Ok(cmd) = cmd_rx.recv() {
                let _ = stdin.write_all(cmd.as_bytes());
                let _ = stdin.flush();
            }
        });

        // stdout reader task
        thread::spawn(move || {
            let mut reader = stdout;
            let mut line = String::new();

            loop {
                line.clear();
                if reader
                    .read_line(&mut line)
                    .ok()
                    .filter(|&n| n > 0)
                    .is_none()
                {
                    break;
                }
                let _ = evt_tx.send(line.clone());
            }
        });

        self.engine_handle = Some(EngineHandle {
            process: child_process,
            tx: cmd_tx,
            rx: evt_rx,
        });

        // EngineHandle {
        //     process: child_process,
        //     tx: cmd_tx,
        //     rx: evt_rx,
        // }
    } //

    pub fn send_command(&mut self, command: &str) {
        if self.engine_handle.is_none() {
            self.spawn_handle();
        }
        self.engine_handle
            .as_ref()
            .unwrap()
            .tx
            .send(command.to_string())
            .ok();
    } //
    pub fn read_line(&self) -> Option<String> {
        if let Some(engine) = self.engine_handle.as_ref() {
            engine.rx.recv().ok()
        } else {
            None
        }
    } //
    pub fn try_read_line(&self) -> Option<String> {
        if let Some(engine) = self.engine_handle.as_ref() {
            engine.rx.try_recv().ok()
        } else {
            None
        }
    } //

    pub fn detect_engine_options(&mut self) -> Vec<EngineOption> {
        if self.engine_handle.is_none() {
            self.spawn_handle();
            // return vec![];
        }
        self.send_command("uci\n");
        let mut options = vec![];
        loop {
            if let Some(str) = self.read_line() {
                if str.starts_with("option") {
                    let args = str.split_whitespace().collect::<Vec<_>>();
                    let option_type;
                    let value;
                    let name;

                    if let Some(name_index) = args.iter().position(|w| w == &"name") {
                        name = args[name_index + 1].to_string();
                    } else {
                        continue;
                    }
                    if let Some(default_index) = args.iter().position(|w| w == &"default") {
                        value = args[default_index + 1].to_string();
                    } else {
                        continue;
                    }
                    if let Some(option_type_index) = args.iter().position(|w| w == &"type") {
                        option_type = args[option_type_index + 1].to_string();
                    } else {
                        continue;
                    }

                    match option_type.as_str() {
                        "check" => {
                            options.push(EngineOption::CHECK {
                                name,
                                value: value.parse::<bool>().unwrap(),
                            });
                        }
                        "spin" => {
                            let mut min = None;
                            let mut max = None;
                            if let Some(min_index) = args.iter().position(|w| w == &"min") {
                                min = Some(args[min_index + 1].parse::<i32>().unwrap());
                            }
                            if let Some(max_index) = args.iter().position(|w| w == &"max") {
                                max = Some(args[max_index + 1].parse::<i32>().unwrap());
                            }
                            options.push(EngineOption::SPIN {
                                name,
                                value: value.parse::<i32>().unwrap(),
                                min,
                                max,
                            });
                        }
                        _ => {}
                    }
                } else if str.contains("uciok") {
                    break;
                }
            } else {
                break;
            }
        }
        options
    } //

    pub fn disconnect(&mut self) {
        if let Some(handle) = self.engine_handle.as_mut() {
            handle.process.kill().ok();
            handle.process.wait().ok();
        }
    }

    pub fn poll_engine(&mut self) {
        if let Some(handle) = self.engine_handle.as_mut() {
            while let Some(line) = handle.try_read_line() {
                if let Some(analysis) = AnalysisLine::new(line) {
                    self.analysis.push(analysis);
                }
            }
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.disconnect();
    }
}

pub struct EngineHandle {
    process: Child,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}
impl Drop for EngineHandle {
    fn drop(&mut self) {
        self.process.kill().ok();
        self.process.wait().ok();
    }
}

impl EngineHandle {
    pub fn send_command(&self, command: &str) {
        self.tx.send(command.to_string()).ok();
    } //
    pub fn read_line(&self) -> Option<String> {
        self.rx.recv().ok()
    } //
    pub fn try_read_line(&self) -> Option<String> {
        self.rx.try_recv().ok()
    }

    pub fn detect_engine_options(&mut self) -> Vec<EngineOption> {
        self.send_command("uci\n");
        let mut options = vec![];
        loop {
            if let Some(str) = self.read_line() {
                if str.starts_with("option") {
                    let args = str.split_whitespace().collect::<Vec<_>>();
                    let option_type;
                    let value;
                    let name;

                    if let Some(name_index) = args.iter().position(|w| w == &"name") {
                        name = args[name_index + 1].to_string();
                    } else {
                        continue;
                    }
                    if let Some(default_index) = args.iter().position(|w| w == &"default") {
                        value = args[default_index + 1].to_string();
                    } else {
                        continue;
                    }
                    if let Some(option_type_index) = args.iter().position(|w| w == &"type") {
                        option_type = args[option_type_index + 1].to_string();
                    } else {
                        continue;
                    }

                    match option_type.as_str() {
                        "check" => {
                            options.push(EngineOption::CHECK {
                                name,
                                value: value.parse::<bool>().unwrap(),
                            });
                        }
                        "spin" => {
                            let mut min = None;
                            let mut max = None;
                            if let Some(min_index) = args.iter().position(|w| w == &"min") {
                                min = Some(args[min_index + 1].parse::<i32>().unwrap());
                            }
                            if let Some(max_index) = args.iter().position(|w| w == &"max") {
                                max = Some(args[max_index + 1].parse::<i32>().unwrap());
                            }
                            options.push(EngineOption::SPIN {
                                name,
                                value: value.parse::<i32>().unwrap(),
                                min,
                                max,
                            });
                        }
                        _ => {}
                    }
                } else if str.contains("uciok") {
                    break;
                }
            } else {
                break;
            }
        }
        options
    } //

    pub fn disconnect(&mut self) {
        self.send_command("quit\n");
    } //
}
