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

#![feature(box_into_inner)]

use std::collections::{HashMap, HashSet};
use grafana_plugin_sdk::data::Field;
use smol::stream::StreamExt;
use {
	std::{
		collections::btree_map,
		sync::Arc,
		borrow::Cow,
		time::Duration
	},
	serde::{*, de::IntoDeserializer},
	grafana_plugin_sdk::{backend::*, data, prelude::*}
};

const KEY_TLS_CERTIFICATE: &str = "tlsCertificate";
const KEY_TLS_PRIVATE_KEY: &str = "tlsPrivateKey";
const KEY_TLS_CA_CERT_KEY: &str = "tlsCACertificate";

#[grafana_plugin_sdk::main(services(data, diagnostics), init_subscriber = true)]
async fn plugin() -> Service {
	Service(Arc::new(ServiceInner {
		data_sources: smol::lock::Mutex::new(btree_map::BTreeMap::new())
	}))
}

#[derive(Deserialize)]
struct DataSourceJsonData {
	url:     String,
	timeout: Option<u64>
}

#[derive(Deserialize)]
#[serde(rename = "camelCase")]
struct QueryData {
	query_text: String
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FieldData {
	U64(Vec<Option<u64>>),
	I64(Vec<Option<i64>>),
	F32(Vec<Option<f32>>),
	F64(Vec<Option<f64>>),
	String(Vec<Option<String>>)
}

#[derive(Clone)]
pub struct Service(Arc<ServiceInner>);

struct ServiceInner {
	data_sources: smol::lock::Mutex<btree_map::BTreeMap<i64, (chrono::DateTime<chrono::Utc>, kranus_db_driver::AsyncClient)>>
}

impl Service {
	async fn get_data_source_connection(&self, settings: &DataSourceInstanceSettings) -> Result<kranus_db_driver::AsyncClient, String> {
		Ok(match self.0.data_sources.lock().await.entry(settings.id) {
			btree_map::Entry::Occupied(entry) if entry.get().0 == settings.updated => entry.get().1.clone(),
			entry => {
				let data = DataSourceJsonData::deserialize(settings.json_data.clone().into_deserializer())
					.map_err(|e| format!("Failed to deserialize data: {}", e))?;

				let url = url::Url::parse(&data.url)
					.map_err(|e| format!("Failed to parse URL: {}", e))?;

				let conn = kranus_db_driver::ClientOptions {
					hostname: url.host().map_or(Cow::Borrowed(kranus_db_driver::DEFAULT_HOST), |v| Cow::Owned(v.to_string())),
					port:     url.port().unwrap_or(kranus_db_driver::DEFAULT_PORT),
					//timeout:  data.timeout.map_or(kranus_db_driver::DEFAULT_TIMEOUT, Duration::from_secs),
					tls:      kranus_db_driver::tls::ClientConfig::builder()
								.with_safe_defaults()
								.with_root_certificates(kranus_db_driver::tls::RootCertStore::empty())
								.with_single_cert(
									vec![kranus_db_driver::tls::Certificate(settings
										.decrypted_secure_json_data
										.get(KEY_TLS_CERTIFICATE)
										.ok_or_else(|| "`tlsCertificate` not present".to_string())?
										.as_bytes()
										.to_vec())],
									kranus_db_driver::tls::PrivateKey(settings
										.decrypted_secure_json_data
										.get(KEY_TLS_PRIVATE_KEY)
										.ok_or_else(|| "`tlsPrivateKey` not present".to_string())?
										.as_bytes()
										.to_vec())
								)
								.map(Arc::new)
								.map_err(|e| format!("Invalid TLS certificate or private key: {}", e))?,
					labels: Vec::new()
				}.connect_async().await
					.map_err(|e| e.to_string())?;

				let val = (settings.updated, conn);
				entry.and_modify(|v| *v = val.clone()).or_insert_with(|| val.clone());
				log::info!("DATASOURCE {} (`{}`): connected with options {}", settings.id, &settings.name, &settings.json_data);
				val.1
			}
		})
	}

	fn get_query(&self, query: DataQuery) -> Result<impl serde::Serialize, String> {
		match QueryData::deserialize(query.json.into_deserializer()) {
			Ok(v)  => query::compile(&v.query_text),
			Err(e) => Err(e.to_string())
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct QueryError {
	ref_id:  String,
	message: String
}

impl std::fmt::Display for QueryError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Error querying backend for {}: {}", &self.ref_id, &self.message)
	}
}

impl std::error::Error for QueryError {}

impl DataQueryError for QueryError {
	fn ref_id(self) -> String {
		self.ref_id
	}
}

#[async_trait::async_trait]
impl DataService for Service {
	type QueryError = QueryError;
	type Iter       = BoxDataResponseIter<Self::QueryError>;

	async fn query_data(&self, request: QueryDataRequest) -> Self::Iter {
		if request.queries.is_empty() {
			return Box::new([].into_iter());
		}

		let conn = match request.plugin_context.datasource_instance_settings.as_ref() {
			Some(v) => self.get_data_source_connection(v).await,
			None    => {
				let msg = "No datasource settings present".to_string();
				return Box::new(request.queries.into_iter().map(move |query| Err(QueryError {
					ref_id:  query.ref_id,
					message: msg.clone()
				})));
			}
		};

		let conn = match conn {
			Ok(v)  => v,
			Err(e) => {
				let msg = format!("Failed to connect to datasource: {}", e);
				return Box::new(request.queries.into_iter().map(move |query| Err(QueryError {
					ref_id:  query.ref_id,
					message: msg.clone()
				})));
			}
		};

		Box::new(smol::stream::iter(request.queries.into_iter())
			.map(move |query| {
				let term = match self.get_query(query) {
					Ok(v)  => v,
					Err(e) => return Err(QueryError {
						ref_id:  query.ref_id,
						message: format!("Failed to parse query: {}", e)
					})
				};

				let cursor = match conn.query::<_, HashMap<String, FieldData>>(term, QueryOptions {
					format:  otel_mdb_driver::ql::Format::Columns,
					.. QueryOptions::default()
				}).await {
					Ok(v)  => v,
					Err(e) => return Err(QueryError {
						ref_id:  query.ref_id,
						message: format!("Failed to execute query: {}", e)
					})
				};

				let data = match cursor.collect::<Result<Vec<_>, _>>().await {
					Ok(v)  => v,
					Err(e) => return Err(QueryError {
						ref_id:  query.ref_id,
						message: format!("Failed to execute query: {}", e)
					})
				};

				match data.into_iter()
					.map(|(key, val)| match val {
						FieldData::U64(v) => v.into_opt_field(key),
						FieldData::I64(v) => v.into_opt_field(key),
						FieldData::F32(v) => v.into_opt_field(key),
						FieldData::F64(v) => v.into_opt_field(key),
						FieldData::String(v) => v.into_opt_field(key),
					})
					.into_frame("frame")
					.check()
				{
					Ok(v)  => Ok(DataResponse::new(query.ref_id, vec![v])),
					Err(e) => Err(QueryError {
						ref_id:  query.ref_id,
						message: format!("Failed to check frame: {:?}", e)
					})
				}
			})
			.collect::<Vec<Fie>>().await
			.into_iter())
	}
}

#[async_trait::async_trait]
impl DiagnosticsService for Service {
	type CheckHealthError    = GenericError;
	type CollectMetricsError = GenericError;

	async fn check_health(&self, request: CheckHealthRequest) -> Result<CheckHealthResponse, Self::CheckHealthError> {
		let plugin_context = match request.plugin_context {
			Some(v) => v,
			None    => return Err(GenericError("No plugin context present".to_string()))
		};

		let conn = match plugin_context.datasource_instance_settings.as_ref() {
			Some(v) => self.get_data_source_connection(v).await,
			None    => return Err(GenericError("No datasource settings present".to_string()))
		};

		let conn = match conn {
			Ok(v)  => v,
			Err(e) => return Ok(CheckHealthResponse {
				status:      HealthStatus::Error,
				message:     e,
				json_details: Default::default()
			})
		};

		Ok(match conn.server().await {
			Ok(_)  => CheckHealthResponse {
				status:      HealthStatus::Ok,
				message:     "OK".to_string(),
				json_details: Default::default()
			},
			Err(e) => CheckHealthResponse {
				status:       HealthStatus::Error,
				message:      e.to_string(),
				json_details: Default::default()
			}
		})
	}

	async fn collect_metrics(&self, request: CollectMetricsRequest) -> Result<CollectMetricsResponse, Self::CollectMetricsError> {
		Ok(CollectMetricsResponse {
			metrics: None
		})
	}
}

pub struct GenericError(String);

impl std::fmt::Debug for GenericError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0)
	}
}

impl std::fmt::Display for GenericError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0)
	}
}

impl std::error::Error for GenericError {}
