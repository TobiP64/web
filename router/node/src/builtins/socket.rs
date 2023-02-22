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
	super::*,
	crate::interfaces::*,
	std::{io, sync::Arc, task::{Poll, Context}, pin::Pin},
	net::{tls, http::{self, traits::AsyncSharedConnectionExt}, utils::AsyncAcceptorExt},
	smol::{io::AsyncWriteExt},
	dyn_error::Result
};

const LOCALHOST: &str = "localhost";

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
	#[serde(flatten)]
	pub socket:     super::ConfigSocket,
	#[serde(default)]
	pub processor: String,
}

pub(super) async fn run(name: &str, cfg: Config) -> Result<()> {
	let id   = crate::component_id(&cfg.processor);
	let name = Arc::new(name.to_string());
	
	match cfg.socket {
		ConfigSocket { tcp: Some(tcp), tls: None, http1: Some(_), .. } => {
			let endpoint = Arc::new(format!(
				"{}:{}",
				tcp.host.as_deref().unwrap_or(LOCALHOST),
				tcp.port.unwrap_or(net::http::v1::DEFAULT_PORT_TLS)
			));
			let processor = crate::get_component::<HttpStreamHandler>(id);
			let telemetry = Arc::new(HttpTelemetry::new(&name, &endpoint));
			let mut acceptor = net::http::v1::AsyncSharedAcceptor::new(
				net::http::v1::AsyncAcceptor::new(
					net::buffered::AsyncAcceptor::new(
							net::tcp::AsyncAcceptor::new(&*endpoint).await?)));
			
			log::info!("frontend `{}` (https://{}): up", &name, &endpoint);
			
			let ctx = (processor, name, endpoint, telemetry);
			crate::spawn(async move {
				loop {
					let (processor, name, endpoint, telemetry) = ctx.clone();
					let f = acceptor.accept().await;
					crate::spawn(async move {
						let conn = match f.await {
							Ok(v) => v,
							Err(e) => {
								log::error!("frontend `{}` (https://{}): failed to accept connection: {}", &name, &endpoint, e);
								return;
							}
						};
						
						http_handle(conn, &name, &endpoint, &*processor, &telemetry).await
					});
				}
			});
		}
		ConfigSocket { tcp: Some(tcp), tls: Some(tls @ ConfigSocketTls { alpn: false, .. }), http1: Some(_), .. } => {
			let endpoint = Arc::new(format!(
				"{}:{}",
				tcp.host.as_deref().unwrap_or(LOCALHOST),
				tcp.port.unwrap_or(net::http::v1::DEFAULT_PORT_TLS)
			));
			let processor = crate::get_component::<HttpStreamHandler>(id);
			let telemetry = Arc::new(HttpTelemetry::new(&name, &endpoint));
			let mut acceptor = net::http::v1::AsyncSharedAcceptor::new(
				net::http::v1::AsyncAcceptor::new(
					net::buffered::AsyncAcceptor::new(
						net::tls::AsyncAcceptor::new(
							net::tcp::AsyncAcceptor::new(&*endpoint).await?,
							tls_config(&tls).await?))));
			
			log::info!("frontend `{}` (https://{}): up", &name, &endpoint);
			
			let ctx = (processor, name, endpoint, telemetry);
			crate::spawn(async move {
				loop {
					let (processor, name, endpoint, telemetry) = ctx.clone();
					let f = acceptor.accept().await;
					crate::spawn(async move {
						let conn = match f.await {
							Ok(v) => v,
							Err(e) => {
								log::error!("frontend `{}` (https://{}): failed to accept connection: {}", &name, &endpoint, e);
								return;
							}
						};
						
						http_handle(conn, &name, &endpoint, &*processor, &telemetry).await
					});
				}
			});
		}
		_ => return Err("invalid config".into())
	}
	
	Ok(())
}

async fn tls_config(cfg: &ConfigSocketTls) -> Result<Arc<tls::r#async::rustls::ServerConfig>> {
	let cert = rustls_pemfile::certs(&mut io::BufReader::new(
		std::fs::File::open(&cfg.certificate)?))
		.with_msg("failed to parse certificate")?
		.into_iter()
		.map(net::tls::r#async::rustls::Certificate)
		.collect::<Vec<_>>();
	
	let key = net::tls::r#async::rustls::PrivateKey(match rustls_pemfile::read_one(
		&mut io::BufReader::new(std::fs::File::open(&cfg.private_key)?))
		.with_msg("failed to parse private key")?
	{
		Some(rustls_pemfile::Item::RSAKey(v)) => v,
		Some(rustls_pemfile::Item::X509Certificate(v)) => v,
		Some(rustls_pemfile::Item::PKCS8Key(v)) => v,
		None => return Err("no private key found in file".into()),
	});
	
	let mut cfg = tls::r#async::rustls::ServerConfig::new(
		Arc::new(tls::r#async::rustls::NoClientAuth));
	cfg.set_single_cert(cert, key)
		.with_msg("bad certificate or private key")?;
	Ok(Arc::new(cfg))
	
	/*Ok(Arc::new(tls::ServerConfig::builder()
		.with_safe_defaults()
		.with_no_client_auth()
		.with_single_cert(cert, key)
		.with_msg("bad certificate or private key")?))*/
}

struct HttpTelemetry {
	connections_accepted:    otel_mrt::BoundInstrument<usize>,
	connections_established: otel_mrt::BoundInstrument<isize>,
	connection_time:         otel_mrt::BoundInstrument<usize>,
	requests_accepted:       otel_mrt::BoundInstrument<usize>,
	requests_in_progress:    otel_mrt::BoundInstrument<isize>,
	request_time:            otel_mrt::BoundInstrument<usize>,
	request_status:          otel_mrt::BoundInstrument<usize>,
	request_body_len:        otel_mrt::BoundInstrument<usize>,
	response_body_len:       otel_mrt::BoundInstrument<usize>
}

impl HttpTelemetry {
	fn new(name: &str, addr: &str) -> Self {
		let rt = otel_mrt::runtime();
		let labels = vec![
			(Cow::Borrowed("module"), otel_mrt::AnyValue::String(name.to_string())),
			(Cow::Borrowed("endpoint"), otel_mrt::AnyValue::String(addr.to_string()))
		];
		
		Self {
			connections_accepted:    rt.instrument::<usize>(otel_mrt::InstrumentParameters::new()
				.name_str("socket_connections_accepted")
				.aggregation_sum(otel_mrt::AggregationTemporality::Unspecified, true))
				.bind(labels.clone()),
			connections_established: rt.instrument::<isize>(otel_mrt::InstrumentParameters::new()
				.name_str("socket_connections_established")
				.aggregation_sum(otel_mrt::AggregationTemporality::Unspecified, true))
				.bind(labels.clone()),
			connection_time:        rt.instrument::<usize>(otel_mrt::InstrumentParameters::new()
				.name_str("socket_connection_lifetime")
				.aggregation_histogram(otel_mrt::AggregationTemporality::Unspecified))
				.bind(labels.clone()),
			requests_accepted:       rt.instrument::<usize>(otel_mrt::InstrumentParameters::new()
				.name_str("socket_streams_accepted")
				.aggregation_sum(otel_mrt::AggregationTemporality::Unspecified, true))
				.bind(labels.clone()),
			requests_in_progress:    rt.instrument::<isize>(otel_mrt::InstrumentParameters::new()
				.name_str("socket_streams_in_progress")
				.aggregation_sum(otel_mrt::AggregationTemporality::Unspecified, true))
				.bind(labels.clone()),
			request_time:            rt.instrument::<usize>(otel_mrt::InstrumentParameters::new()
				.name_str("socket_stream_lifetime")
				.aggregation_histogram(otel_mrt::AggregationTemporality::Unspecified))
				.bind(labels.clone()),
			request_status:          rt.instrument::<usize>(otel_mrt::InstrumentParameters::new()
				.name_str("socket_stream_http_status")
				.aggregation_histogram(otel_mrt::AggregationTemporality::Unspecified))
				.bind(labels.clone()),
			request_body_len:        rt.instrument::<usize>(otel_mrt::InstrumentParameters::new()
				.name_str("socket_stream_http_request_body_len")
				.aggregation_histogram(otel_mrt::AggregationTemporality::Unspecified))
				.bind(labels.clone()),
			response_body_len:       rt.instrument::<usize>(otel_mrt::InstrumentParameters::new()
				.name_str("socket_stream_http_response_body_len")
				.aggregation_histogram(otel_mrt::AggregationTemporality::Unspecified))
				.bind(labels.clone())
		}
	}
}

#[allow(clippy::needless_lifetimes)]
async fn http_handle<T: http::traits::AsyncSharedConnection>(
	connection: T,
	name:       &str,
	endpoint:   &str,
	processor:  &HttpStreamHandler,
	telemetry:  &HttpTelemetry
) {
	let conn_start = std::time::Instant::now();
	telemetry.connections_accepted.record(1);
	telemetry.connections_established.record(1);
	log::trace!("frontend `{}` (https://{}): connection established", name, endpoint);
	
	let r = loop {
		let (id, stream) = match connection.opened().await {
			Ok(Some(id)) => (id, http::AsyncStream::new(&connection, id)),
			Ok(None) => break Ok(()),
			Err(ref e) if e.kind() == io::ErrorKind::TimedOut => break Ok(()),
			Err(e) => break Err(e)
		};
		
		let start = std::time::Instant::now();
		let mut stream: StreamInterceptor<http::AsyncStream<'_, T>> = StreamInterceptor::new(stream);
		let stream_static = unsafe { std::mem::transmute::<
			&'_      mut (dyn http::traits::AsyncStream + '_),
			&'static mut (dyn http::traits::AsyncStream + 'static)
		>(&mut stream as &mut dyn http::traits::AsyncStream) };
		
		telemetry.requests_accepted.record(1);
		telemetry.requests_in_progress.record(1);
		
		let mut r = processor.accept(stream_static).await;
		
		if r.is_ok() {
			r = stream.close().await.map_err(Into::into);
		}
		
		telemetry.request_time.record(start.elapsed().as_millis() as _);
		telemetry.requests_in_progress.record(-1);
		
		if let Some(status) = stream.status {
			telemetry.request_status.record(status as _);
		}
		
		match r {
			Ok(()) => log::info!(
				"frontend `{}` (https://{}): #{} {} {} -> {} ({} ms)",
				name,
				endpoint,
				id,
				stream.method.map_or_else(|| "?".to_string(), |v| v.to_string()),
				stream.path.unwrap_or_else(|| "?".to_string()),
				stream.status.map_or_else(|| "?".to_string(), |v| v.to_string()),
				start.elapsed().as_millis()
			),
			Err(e) if e.is::<io::Error>() => log::error!(
				"frontend `{}` (https://{}): #{} error: {}",
				name,
				endpoint,
				id,
				e.display()
			),
			Err(e) => log::error!(
				"frontend `{}` (https://{}): #{} {} {} -> {} ({} ms): {}",
				name,
				endpoint,
				id,
				stream.method.map_or_else(|| "?".to_string(), |v| v.to_string()),
				stream.path.unwrap_or_else(|| "?".to_string()),
				stream.status.map_or_else(|| "?".to_string(), |v| v.to_string()),
				start.elapsed().as_millis(),
				e.display()
			)
		}
	};
	
	telemetry.connection_time.record(conn_start.elapsed().as_millis() as _);
	telemetry.connections_established.record(-1);
	
	match r {
		Ok(()) => log::trace!("frontend `{}` (https://{}): connection closed ({} ms)",
			name, endpoint, conn_start.elapsed().as_millis()),
		Err(e) => log::error!("frontend `{}` (https://{}): connection aborted ({} ms): {}",
			name, endpoint, conn_start.elapsed().as_millis(), e),
	}
}

struct StreamInterceptor<T: http::traits::AsyncStream> {
	inner:  T,
	method: Option<http::Method>,
	path:   Option<String>,
	status: Option<http::Status>
}

impl<T: http::traits::AsyncStream> StreamInterceptor<T> {
	fn new(inner: T) -> Self {
		Self {
			inner,
			method: None,
			path:   None,
			status: None
		}
	}
	
	fn set_headers(&mut self, headers: &[http::Header]) {
		for header in headers {
			match header {
				http::Header::Method(v) => self.method = Some(v.clone()),
				http::Header::Path(v)   => self.path   = Some(v.clone()),
				http::Header::Status(v) => self.status = Some(*v),
				_ => ()
			}
		}
	}
}

impl<T: http::traits::AsyncStream> http::traits::AsyncStream for StreamInterceptor<T> {
	fn poll_read_headers<'a>(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Vec<http::Header>>> {
		let self_ = unsafe { Pin::into_inner_unchecked(self) };
		match unsafe { Pin::new_unchecked(&mut self_.inner) }.poll_read_headers(cx) {
			Poll::Ready(Ok(headers)) => {
				self_.set_headers(&headers);
				Poll::Ready(Ok(headers))
			},
			v => v
		}
	}
	
	fn poll_write_headers(self: Pin<&mut Self>, cx: &mut Context<'_>, headers: &[http::Header]) -> Poll<io::Result<()>> {
		let self_ = unsafe { Pin::into_inner_unchecked(self) };
		self_.set_headers(headers);
		unsafe { Pin::new_unchecked(&mut self_.inner) }.poll_write_headers(cx, headers)
	}
}

impl<T: http::traits::AsyncStream> smol::io::AsyncWrite for StreamInterceptor<T> {
	fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_write(cx, buf)
	}
	
	fn poll_write_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[io::IoSlice<'_>]) -> Poll<std::io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_write_vectored(cx, bufs)
	}
	
	fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
		unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_flush(cx)
	}
	
	fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
		unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_close(cx)
	}
}

impl<T: http::traits::AsyncStream> smol::io::AsyncRead for StreamInterceptor<T> {
	fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_read(cx, buf)
	}
	
	fn poll_read_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &mut [io::IoSliceMut<'_>]) -> Poll<std::io::Result<usize>> {
		unsafe { self.map_unchecked_mut(|v| &mut v.inner) }.poll_read_vectored(cx, bufs)
	}
}
