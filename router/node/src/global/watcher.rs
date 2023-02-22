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

use {
	std::{sync::{Arc, mpsc}, time::Duration},
	notify::{DebouncedEvent, RecommendedWatcher}
};

pub const DEFAULT_WATCHER_DELAY: Duration = Duration::from_secs(1);

pub async fn run(ctx: Arc<crate::ctx::ContextWrapper>, tx: mpsc::Receiver<DebouncedEvent>, watcher: RecommendedWatcher) {
	log::info!("watcher: listening for changes");
	
	for event in tx.iter() {
		match event {
			DebouncedEvent::NoticeWrite(_)  => (),
			DebouncedEvent::NoticeRemove(_) => (),
			DebouncedEvent::Create(path) => {
			
			}
			DebouncedEvent::Write(path) => {
			
			}
			DebouncedEvent::Chmod(path) => {
			
			}
			DebouncedEvent::Remove(path) => {
				let mut inner = ctx.0.lock().await;
				match inner.files.remove(&path) {
					Some(_) => log::info!("watcher: removed `{}`", path.display()),
					None    => log::warn!("watcher: received remove event for `{}`, but this path is not registered", path.display())
				}
			}
			DebouncedEvent::Rename(path, new) => {
				let mut inner = ctx.0.lock().await;
				let v = match inner.files.remove(&path) {
					Some(v) => v,
					None    => {
						log::warn!("watcher: received remove event for `{}`, but this path is not registered", path.display());
						continue;
					}
				};
				
				log::info!("watcher: renamed `{}` to `{}`", path.display(), new.display());
				inner.files.insert(new, v);
			}
			DebouncedEvent::Rescan => {
				log::info!("watcher: rescan triggered");
			}
			DebouncedEvent::Error(e, path) => match path {
				None       => log::error!("watcher: {}", e),
				Some(path) => log::error!("watcher: `{}`: {}", path.display(), e)
			}
		}
	}
}