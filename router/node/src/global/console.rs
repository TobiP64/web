// MIT License
//
// Copyright (c) 2019-2023 Tobias Pfeiffer
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use super::*;

const HELP: &str = r#"
?, h, help         show this help page
s, show <key>      show information
get <key>          show configuration
set <key> <val>    set configuration
r, reload          reload configuration
exit               shut down and exit
abort              abort the process
"#;

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields, from = "ConfigEnum<Config>")]
pub struct Config {
	pub enabled: bool
}

impl From<ConfigEnum<Config>> for Config {
	fn from(v: ConfigEnum<Config>) -> Self {
		match v {
			ConfigEnum::Bool(false) => Self::default(),
			ConfigEnum::Bool(true)  => Self { enabled: true, ..Self::default() },
			ConfigEnum::Config(cfg) => cfg
		}
	}
}

#[allow(unused_must_use)]
pub fn run(cfg: Config) {
	let stdin = std::io::stdin();
	let mut buf = String::new();

	loop {
		{
			let stdout = std::io::stdout();
			let mut stdout = stdout.lock();
			stdout.write_all(b"> ");
			stdout.flush();
			std::mem::drop(stdout);
		}

		buf.clear();

		while let Err(e) = stdin.read_line(&mut buf) {
			buf.clear();
			log::error!("global: failed to read from stdin: {}, retrying in 10 seconds", e);
			std::thread::sleep(std::time::Duration::from_secs(10));
		}

		let args = buf.split_ascii_whitespace().collect::<Vec<_>>();

		match args.as_slice() {
			[] => (),
			["?" | "h" | "help", ..] => println!("{}", HELP),
			["s" | "show", tail @ ..] => match tail {
				["pid"] => println!("cli: PID: {}", std::process::id()),
				["dir"] => println!("cli: WORKING DIR: {}", match std::env::current_dir() {
					Ok(dir) => dir.to_string_lossy().into_owned(),
					Err(e) => e.to_string()
				}),
				["cfg"] => println!("cli: CONFIG:\n{:#?}\n", &cfg),
				_ => eprintln!("show: invalid option")
			}
			["get", tail @ ..] => match tail {
				[key] => {
					for key in key.split('.') {

					}
				}
				_ => eprintln!("get: expecting 1 parameter")
			}
			["set", tail @ ..] => match tail {
				[key, val] => {
					for key in key.split('.') {

					}
				}
				_ => eprintln!("set: expecting 2 parameters")
			}
			["reload", ..] =>  {

			}
			["stop", ..] => {
				println!("Stopping ...");
				std::process::exit(0);
			}
			["abort", ..] => {
				println!("Aborting ...");
				std::process::abort();
			}
			["log", tail @ ..] => match tail {
				["e" | "error", tail @ ..] => log::error!("cli: {}", tail.join(" ")),
				["w" | "warn", tail @ ..]  => log::warn!("cli: {}", tail.join(" ")),
				["i" | "info", tail @ ..]  => log::info!("cli: {}", tail.join(" ")),
				["d" | "debug", tail @ ..] => log::debug!("cli: {}", tail.join(" ")),
				["t" | "trace", tail @ ..] => log::trace!("cli: {}", tail.join(" ")),
				_ => eprintln!("log: invalid option")
			}
			_ => eprintln!("unknown or invalid command")
		}
	}
}