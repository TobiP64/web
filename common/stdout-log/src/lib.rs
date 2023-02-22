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

#![warn(clippy::all)]
#![forbid(unsafe_code)]

static LOGGER: Logger = Logger;

pub fn get() -> &'static Logger {
    &LOGGER
}

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    
    fn log(&self, record: &log::Record) {
        use {std::io::Write, log::Level::*};
        
        let thread = std::thread::current();
        let thread = thread.name().unwrap_or("unnamed");
        let date   = chrono::Utc::now().format("%F %T%.3f");
        let target = format!("{}:{}", record.target(), record.line().unwrap_or(0));
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        
        match record.level() {
            Trace => writeln!(&mut stdout, "\x1b[90m{} \x1b[90;7mTRACE\x1b[27;90m {:48} {:16} {}\x1b[0m",
                             date, target, thread, record.args()),
            Debug => writeln!(&mut stdout, "\x1b[32m{} \x1b[32;7mDEBUG\x1b[27;32m {:48} {:16} {}\x1b[0m",
                             date, target, thread, record.args()),
            Info  => writeln!(&mut stdout, "\x1b[0m{} \x1b[7mINFO\x1b[27m  {:48} {:16} {}\x1b[0m",
                             date, target, thread, record.args()),
            Warn  => writeln!(&mut stdout, "\x1b[33m{} \x1b[33;7mWARN\x1b[27;33m  {:48} {:16} {}\x1b[0m",
                             date, target, thread, record.args()),
            Error => writeln!(&mut stdout, "\x1b[31m{} \x1b[31;7mERROR\x1b[27;31m {:48} {:16} {}\x1b[0m",
                             date, target, thread, record.args()),
        }.unwrap_or(());
    }
    
    fn flush(&self) {}
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[ignore]
    #[test]
    fn log() {
        log::set_max_level(log::LevelFilter::Trace);
        log::set_logger(get()).unwrap();
        
        log::trace!("test trace");
        log::debug!("test debug");
        log::info!("test info");
        log::warn!("test warn");
        log::error!("test error");
    }
}
