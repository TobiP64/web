// MIT License
//
// Copyright (c) 2022 Tobias Pfeiffer
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

use std::fs::File;
use std::io;
use std::path::Path;
use nix::fcntl::OFlag;
use nix::sys::mman::{MapFlags, ProtFlags};
use nix::sys::stat::Mode;
use fs2::FileExt;

const ENV_CFG:       &str = "KRANUS_DB_CFG";
const ENV_DAT:       &str = "KRANUS_DB_DAT";
const ENV_LOG:       &str = "KRANUS_DB_LOG";
const ENV_LOG_LEVEL: &str = "KRANUS_DB_LOG_LEVEL";
const DEFAULT_CFG:   &str = "./config.toml";
const DEFAULT_DAT:   &str = "./data.bin";
const DEFAULT_LOG:   &str = "./logs.bin";
const DEFAULT_LOG_LEVEL: usize = 1;

pub async fn startup(
	cfg_path:  Option<&Path>,
	dat_path:  Option<&Path>,
	log_path:  Option<&Path>,
	log_level: Option<usize>
) -> io::Result<super::Context> {
	log::info!("init: commencing initialization sequence ...");
	let t = std::time::Instant::now();
	
	let cfg_path = cfg_path.unwrap_or(Path::new(DEFAULT_CFG));
	let dat_path = dat_path.unwrap_or(Path::new(DEFAULT_DAT));
	let log_path = log_path.unwrap_or(Path::new(DEFAULT_LOG));
	
	log::info!("init [1/8] loading config ...");
	
	log::info!("init [1/8] loading config: done");
	log::info!("init [2/8] loading logs ...");
	
	
	log::info!("init [2/8] loading logs: done");
	log::info!("init [3/8] checking logs ...");
	
	
	log::info!("init [3/8] checking logs: done");
	log::info!("init [4/8] loading data ...");
	
	let fd = nix::fcntl::open(
		"./data.bin",
		OFlag::O_ASYNC | OFlag::O_CREAT | OFlag::O_LARGEFILE | OFlag::O_NONBLOCK,
		Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP
	).expect("failed to map persistent data");

	unsafe { nix::sys::mman::mmap(
		crate::storage::VERSION_BASE as _,
		(crate::storage::VERSION_SIZE - crate::storage::VERSION_TEMP) as _,
		ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
		MapFlags::MAP_SHARED | MapFlags::MAP_FIXED | MapFlags::MAP_NORESERVE,
		fd,
		0
	).expect("failed to map temporary data"); }
	
	log::info!("init [4/8] loading data: done");
	log::info!("init [5/8] checking data ...");
	
	
	log::info!("init [5/8] checking data: done");
	log::info!("init [6/8] reading metadata ...");
	
	
	log::info!("init [6/8] reading metadata: done");
	log::info!("init [7/8] initializing telemetry endpoint ...");
	
	
	log::info!("init [7/8] initializing telemetry endpoint: done");
	log::info!("init [8/8] initializing endpoint ...");
	
	
	log::info!("init [8/8] initializing endpoint: done");
	log::info!("init [9/8] connecting to cluster ...");
	
	
	log::info!("init [9/8] connecting to cluster: done");

	Ok(super::Context {
		data:     0,
		log:      (),
		memory:   super::storage::StorageService::,
		topology: ()
	})
}