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

pub mod ring_buffer;
pub mod pipe;
pub mod headers;
#[cfg(feature = "smol")]
pub mod scheduler;
pub mod unstable;
pub mod connection;

use {std::{pin::Pin, task::{Context, Poll}}};

pub use {connection::*, headers::*, ring_buffer::*};

#[cfg(feature = "smol")]
pub use scheduler::*;

pub type Url = String;

const NTP_UNIX_SECS_DIFF: i64 = 2208988800;

pub fn ntp_to_date_time(v: i64) -> chrono::DateTime<chrono::Utc> {
	chrono::DateTime::from_utc(chrono::naive::NaiveDateTime::from_timestamp_opt(
		v - NTP_UNIX_SECS_DIFF, 0).expect("timestamp is out of range"), chrono::Utc)
}

pub fn date_time_to_ntp(v: chrono::DateTime<chrono::Utc>) -> i64 {
	v.timestamp() + NTP_UNIX_SECS_DIFF
}

pub fn get_scheme_default_port(scheme: &str) -> Option<u16> {
	Some(match scheme {
		"http"  => crate::http::v1::DEFAULT_PORT,
		"https" => crate::http::v1::DEFAULT_PORT_TLS,
		"ldap"  => crate::ldap::DEFAULT_PORT,
		"ldaps" => crate::ldap::DEFAULT_PORT_TLS,
		"rtsp"  => crate::rtsp::DEFAULT_PORT,
		"rtsps" => crate::rtsp::DEFAULT_PORT_TLS,
		"smtp"  => crate::smtp::DEFAULT_PORT,
		"smtps" => crate::smtp::DEFAULT_PORT_TLS,
		"ws"    => crate::ws::DEFAULT_PORT,
		"wss"   => crate::ws::DEFAULT_PORT_TLS,
		_ =>       return None
	})
}

/// Executes all futures yielded by `futures` and returns the results out-of-order as a stream.
pub fn zip<'a, I: IntoIterator<Item = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>>, T>(futures: I) -> ZipStream<'a, T> {
	ZipStream(futures.into_iter().enumerate().collect())
}

#[allow(clippy::type_complexity)]
pub struct ZipStream<'a, T>(Vec<(usize, Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>)>);

impl<'a, T> futures_lite::Stream for ZipStream<'a, T> {
	type Item = (usize, T);

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let Self(futures) = &mut*self;

		if futures.is_empty() {
			return Poll::Ready(None);
		}

		for (i, (j, f)) in futures.iter_mut().enumerate() {
			if let Poll::Ready(v) = f.as_mut().poll(cx) {
				let __j__ = *j;
				futures.swap_remove(i);
				return Poll::Ready(Some((__j__, v)));
			}
		}

		Poll::Pending
	}
}