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
	std::str::FromStr,
	net::{http::{self, traits::*}, utils::*}
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Protocol {
	Grpc,
	HttpJson,
	HttpProto,
}

impl std::str::FromStr for Protocol {
	type Err = ();
	
	fn from_str(v: &str) -> Result<Self, Self::Err> {
		Ok(match v {
			"grpc"          => Self::Grpc,
			"http/json"     => Self::HttpJson,
			"http/protobuf" => Self::HttpProto,
			_               => return Err(())
		})
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Compression {
	GZip
}

impl std::str::FromStr for Compression {
	type Err = ();
	
	fn from_str(v: &str) -> Result<Self, Self::Err> {
		Ok(match v {
			"gzip" => Self::GZip,
			_      => return Err(())
		})
	}
}

const IO_BUF_LEN: usize = 0x1000;

pub(crate) async fn run(mut runtime: Arc<RuntimeInner>) {
	if runtime.config.disabled {
		return;
	}
	
	let connector: net::http::traits::BoxedAsyncConnector = match &runtime.config.tls_config {
		None      => Box::pin(net::http::traits::DynAsyncConnector::new(net::http::v1::connector_async(
			runtime.config.endpoint.to_string()))),
		Some(cfg) => {
			let name = net::tls::r#async::webpki::DNSNameRef::try_from_ascii_str(runtime.config.endpoint
				.split_once(':')
				.map_or("", |(v, _)| v))
				.expect("invalid DNS name")
				.to_owned();
			
			Box::pin(net::http::traits::DynAsyncConnector::new(net::http::v1::connector_tls_async(
				runtime.config.endpoint.to_string(), name, cfg.clone())))
		}
	};
	
	let mut connection = None;
	let mut buf = Vec::with_capacity(IO_BUF_LEN);
	
	log::info!("[OpenTelemetry Exporter] exporter started");
	
	loop {
		smol::Timer::after(runtime.config.interval).await;
		
		match Arc::try_unwrap(runtime) {
			Ok(_) => {
				log::info!("[OpenTelemetry Exporter] shutting down");
				return;
			}
			Err(v) => runtime = v
		}
		
		let conn = match connection.as_mut() {
			Some(v) => v,
			None => {
				let t = std::time::Instant::now();
				match connector.connect().await {
					Ok(conn) => {
						log::info!("[OpenTelemetry Exporter] connected to endpoint ({}ms)", t.elapsed().as_millis());
						connection.insert(conn)
					}
					Err(e) => {
						log::error!("[OpenTelemetry Exporter] failed to connect to endpoint: {}", e);
						continue;
					}
				}
			}
		};
		
		{
			let data = runtime.sync.lock().expect("failed to lock runtime");
			
			// TODO prepare data
		}
		
		let r = (async {
			let len = 0;
			
			let stream_metrics = conn.open().await?;
			conn.write_headers(stream_metrics, &http_headers(net::otlp::metrics::HTTP_PATH, len)).await?;
			conn.write_body(stream_metrics, &buf[..len]).await?;
			
			let stream_tracing = conn.open().await?;
			conn.write_headers(stream_tracing, &http_headers(net::otlp::tracing::HTTP_PATH, len)).await?;
			conn.write_body(stream_tracing, &buf[..len]).await?;
			
			let stream_logging = conn.open().await?;
			conn.write_headers(stream_logging, &http_headers(net::otlp::logging::HTTP_PATH, len)).await?;
			conn.write_body(stream_logging, &buf[..len]).await?;
			
			while let Some(v) = conn.read(&mut buf).await? {
				match v {
					(_,  http::Read::Opened)      => return Err(std::io::Error::new(std::io::ErrorKind::Other, "server opened push stream")),
					(id, http::Read::Headers(v))  => todo!(),
					(id, http::Read::HeadersDone) => todo!(),
					(id, http::Read::Body(v))     => todo!(),
					(id, http::Read::Closed)      => todo!()
				}
			}
			
			std::io::Result::Ok(())
		}).await;
		
		if let Err(e) = r {
			log::warn!("[OpenTelemetry Exporter] failed to send data: {}, terminating connection", e);
			connection.take();
		}
	}
}

fn http_headers(path: &str, length: usize) -> [http::Header; 5] {
	[
		http::Header::Method(http::Method::Post),
		http::Header::Path(path.to_string()),
		http::Header::UserAgent("otel-mrt".to_string()),
		http::Header::ContentType(Box::new(MediaType::from_str("application/json").unwrap())),
		http::Header::ContentLength(length)
	]
}

pub fn get_default_executor() -> Box<Executor> {
	Box::new(|task| { std::thread::spawn(move || smol::block_on(task)); })
}