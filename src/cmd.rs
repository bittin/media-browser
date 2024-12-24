
use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Command, Stdio},
    thread::JoinHandle,
};

pub struct CmdRunner {
    pub cmd: Command,
}

fn tee<R, W>(reader: R, mut writer: W) -> JoinHandle<io::Result<Vec<String>>>
where
    R: BufRead + Send + 'static,
    W: Write + Send + 'static,
{
    std::thread::spawn(move || {
        let mut capture = Vec::new();

        for line in reader.lines() {
            let line = line?;

            capture.push(line.clone());
            writer.write_all(line.as_bytes())?;
        }

        Ok(capture)
    })
}

impl CmdRunner {
    pub fn new(cmd_str: &str) -> CmdRunner {
        let mut cmd = Command::new("script");

        cmd.arg("-qec").arg(cmd_str).arg("/dev/null");
        CmdRunner { cmd }
    }

    pub fn run_with_output(&mut self) -> io::Result<(Vec<String>, Vec<String>)> {
        self.cmd.stdin(Stdio::null());
        self.cmd.stdout(Stdio::piped());
        self.cmd.stderr(Stdio::piped());

        let mut child = self.cmd.spawn().expect("failed to spawn command");
        let stdout_pipe = child.stdout.take().unwrap();
        let stdout_thread = tee(BufReader::new(stdout_pipe), io::stdout());
        let stderr_pipe = child.stderr.take().unwrap();
        let stderr_thread = tee(BufReader::new(stderr_pipe), io::stderr());
        let stdout_output = stdout_thread.join().expect("failed to join stdout thread");
        let stderr_output = stderr_thread.join().expect("failed to join stderr thread");

        Ok((
            stdout_output?,
            stderr_output?,
        ))
    }
}

