use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

struct Shell {
    prompt: String,
}

impl Shell {
    fn new(prompt: impl Into<String>) -> Self {
        Self { prompt: prompt.into() }
    }

    fn run(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        loop {
            // Print prompt
            print!("{}", self.prompt);
            stdout.flush()?; // ensure prompt shows before read

            let mut line = String::new();
            let bytes = io::stdin().read_line(&mut line)?;

            // bytes == 0 => EOF (Ctrl-D). Exit cleanly.
            if bytes == 0 {
                println!();
                return Ok(());
            }

            let cmd_line = line.trim();
            match cmd_line {
                "" => continue,
                "exit" => return Ok(()),
                _ => self.run_cmdline(cmd_line)?,
            }
        }
    }

    fn run_cmdline(&self, cmd_line: &str) -> io::Result<()> {
        // simple whitespace split; (no shell quoting)
        let mut parts = cmd_line.split_whitespace();
        if let Some(program) = parts.next() {
            let args: Vec<&str> = parts.collect();
            self.run_cmd(program, &args)?;
        }
        Ok(())
    }

    fn run_cmd(&self, program: &str, argv: &[&str]) -> io::Result<()> {
        // Built-ins (example: cd)
        if program == "cd" {
            // cd with no args -> home, else first arg
            let target = argv
                .get(0)
                .map(|s| s.to_string())
                .or_else(|| env::var("HOME").ok())
                .or_else(|| env::var("USERPROFILE").ok()); // Windows fallback
            match target {
                Some(path) => {
                    if let Err(e) = env::set_current_dir(&path) {
                        eprintln!("cd: {}: {}", path, e);
                    }
                }
                None => eprintln!("cd: no target directory"),
            }
            return Ok(());
        }

        // No need to check with `which`; Command searches PATH for us.
        let status = Command::new(program)
            .args(argv)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        match status {
            Ok(s) => {
                if !s.success() {
                    // optional: show exit code for debugging
                    if let Some(code) = s.code() {
                        eprintln!("{program}: exited with status {code}");
                    } else {
                        eprintln!("{program}: terminated by signal");
                    }
                }
            }
            Err(err) => {
                eprintln!("{program}: {err}");
            }
        }

        Ok(())
    }
}

fn parse_dash_c() -> Option<String> {
    // Minimal std-only parsing for: gash -c "cmd here"
    let args: Vec<String> = env::args().collect();
    let mut iter = args.iter();
    // skip program name
    iter.next();

    while let Some(a) = iter.next() {
        if a == "-c" {
            return iter.next().cloned();
        }
    }
    None
}

fn main() -> io::Result<()> {
    if let Some(cmd_line) = parse_dash_c() {
        Shell::new("").run_cmdline(&cmd_line)
    } else {
        Shell::new("gash > ").run()
    }
}
