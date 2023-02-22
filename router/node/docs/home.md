# net-services

## Installation

### Build from Source

### Cargo

### Container Images

### Packages

### Repositories

#### RHEL/CentOS/Fedora

#### Debian/Ubuntu

## Features

### Monitoring

### Console

## Configuration

#### File Discovery

#### Include Files

#### Environment Variables

#### Global

### Builtin Modules

net-services provides several builtin modules that can be used without loading a plugin.

#### Frontend

| Field      | Type   | Description
|:-----------|:-------|:---
| chain_next | String | The next module in the chain.

#### Balancer

| Field           | Type   | Description
|:----------------|:-------|:---
| backends        | Array  | A list of backends.
| backends.name   | String | The name of a module.
| backends.weight | Float  | The weight of the backend, backends with a higher weight are preferred when routing requests.
| method          | Enum   | Specifies how to distribute requests over the backends. One of `leastconn`, `fastest`, `random`, `roundrobin`, `source`.

#### Cache

| Field      | Type   | Description
|:-----------|:-------|:---
| chain_next | String | The next module in the chain.
| path       | Path   | The cache's directory, if omitted, the cache is in-memory only.
| expiration | Int    | After which time a resource is considered outdated.
| revalidate | Bool   | Enables revalidation with the origin server for each request.
| min_uses   | Int    | The number of times a resource must be requeted before it is cached.
| method     | Enum   | The method that is used to decide which resource to evict when out of memory. One of `LRU`, `LFU`, `MRU`, `ARC`.
| size       | Int    | The size of the cache.
| preload    | Bool   | If `chain_next` refers to a Storage module, the cache is populated with all files.

#### Router

#### Auth

| Field      | Type   | Description
|:-----------|:-------|:---
| chain_next | String | The next module in the chain.
| source     | Array  |
| scheme     | Array  |

#### Storage

| Field             | Type   | Description
|:------------------|:-------|:---
| path              | Path   | The directory to index.
| writable          | bool   | Enables write access
| recursive         | bool   | Enables recursive indexing of all sub directories.
| filter.allow_list | Array  | An array of expressions. Only file names that match at least one expression will be loaded.
| filter.deny_list  | Array  | An array of expressions. Only file names that do not match any expression will be loaded.
| http.etag         | String | Specifies how the E-tag of a file is generated for HTTP requests. Available options are `SHA-3` and `XXH-3`.

#### Relay

| Field           | Type | Description
|:----------------|:-----|:---
| retries         | Int  | Specifies how many reconnects are attempted before considering a server unavailable.
| retry_interval  | Int  | The interval for retries.
| retry_backoff   | Int  |
| buf_len         | Int  | The length of the IO buffer that is allocated for each request.
| check           | Bool |

### Examples

## Plugins

### Structure

```rust
plugin! {
    <component name>: <component struct> as <interface1> + <interface2> ...,
    ...
};
```

```rust
#![warn(clippy::all)]

extern crate kranus_protocols as net;

use {
	net::*,
	kranus_router_api::{*, interfaces::*},
	kranus_router_controller::*,
	serde::*
};

plugin! {
    plugin: Context as HttpStreamHandler
};

#[derive(Debug, Deserialize)]
struct Config {

}

struct Context {

}

#[routes(ArcContext)]
impl Context {
	async fn new<'a>(cfg: &mut (dyn dyn_serde::Deserializer<'a> + Send + Sync)) -> net_services::Result<Self> {
		let cfg = Config::deserialize(cfg).unwrap();
		dbg!(cfg);
		Ok(Self {})
	}

	#[route(method = "GET", path = "/api/health")]
	async fn get_health(&self, #[stream] stream: &mut dyn http::traits::AsyncStream) -> net_services::Result<()> {
		let response = "OK";

		http::MessageBuilder::new()
			.status(http::Status::Ok)
			.content_length(response.len())
			.body(response.as_bytes())
			.send_async(stream)
			.await.map_err(Into::into)
	}

	#[route]
	async fn invalid_route(&self, #[stream] stream: &mut dyn http::traits::AsyncStream) -> net_services::Result<()> {
		http::MessageBuilder::new()
			.status(http::Status::NotFound)
			.content_length(0)
			.send_async(stream)
			.await.map_err(Into::into)
	}
}
```

### Configuration

## Comparisons

### Static Files



### Load Balancer

#### net-services

```toml
[frontend1]
type      = "frontend"
tcp.addr  = "[::]"
tcp.port  = 80
http1     = {}
processor = "backend1"

[backend1]
type     = "balancer"
method   = "roundrobin"
backends = ["server1", "server2"]

[server1]
type      = "relay"
tcp.addr  = "www1.example.com"
tcp.port  = 80
http1     = {}

[server2]
type      = "relay"
tcp.addr  = "www2.example.com"
tcp.port  = 80
http1     = {}
```

#### HAProxy

```
frontend frontend1
  bind *:80
  mode http
  default_backend backend1

backend backend1
   balance roundrobin
   server server1 www1.example.com:80 check
   server server2 www2.example.com:80 check
```

#### nginx

```
http {
    upstream backend1 {
        server www1.example.com;
        server www2.example.com;
    }

    server {
        listen 80;

        location / {
            proxy_pass http://backend1;
        }
    }
}

```

### Apache

```
<VirtualHost *:80>
...
   <Proxy balancer://backend1>
      BalancerMember http://www1.example.com:80
      BalancerMember http://www2.example.com:80
    </Proxy>
    ProxyPreserveHost On
    ProxyPass / balancer://backend1/
    ProxyPassReverse / balancer://backend1/
...
</VirtualHost>
```

## Benchmarks