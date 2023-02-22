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

#![warn(clippy::all, unsafe_code)]

extern crate net_protocols as net;

mod tcp;
mod udp;
mod http;
mod smtp;
mod dns;

use std::{
	io::{self, Read, Write, BufRead},
	time::Duration,
	str::FromStr
};

type Url = String;

const VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

const HELP: &str = r#"
Usage: net-cli <url> <request parameters> [options...]

Description:
 Command line utility to interact with a server, using protocols like HTTP, SMTP and DNS.

URL Scheme                            Request Parameters
 tcp, udp                              -
 dns, dot, https                       <DOMAIN> [Q-TYPE...] [Q-CLASS]
 http, https                           [HTTP-METHOD]
 smtp, smpts                           <MAIL FROM> <RCPT TO>...
 ldap, ldaps                           <OP>

Options:
 General:
  -V, --version                         Show the binary's version and exit
  -h, --help                            Show this help page and exit
      --timeout <TIMEOUT>               Total request timeout

 Input/Output Data:
  -d, --data <FILE>                     Data for HTTP POST/PUT/PATCH requests
  -f, --input-file <FILE|->             Input file
      --input-format <FILE>             Input format
  -o, --output <FILE|->                 Output file
      --output-format <FILE>            Output format, may be RAW|JSON|HUMAN
  -a, --append <FILE>                   Append output to file

 Domain Name Resolution:
  -D, --dns <URL>                       Resolve host names over this DNS server, supports DoH (https://) and DoT (dot://), may be specified multiple times

 TCP/UDP:
  -I, --interface <NAME>                Use a network interface
  -S, --socket <IP[:<PORT|RANGE>]>      The socket address to bind (UDP only)
      --conn-timeout <SECONDS>          Timeout for connecting to the peer (TCP only)
      --ttl <TTL>                       Time to live for the TCP socket
      --nodelay                         Data is sent as soon as possible (TCP only)
      --keep-alive <SECONDS>            Enables keep-alive messages to be sent on the socket (TCP only)
      --read-timeout <SECONDS>          Timeout for reading from the stream
      --write-timeout <SECONDS>         Timeout for writing to the stream
      --recv-buffer <BUFFER SIZE>       The operating system's receive buffer size
      --send-buffer <BUFFER SIZE>       The operating system's send buffer size

 TLS:
  -c, --cert <FILE[:PASSWORD]>          TLS client certificate
  -k, --privkey <FILE[:PASSWORD]>       TLS client private key
  -K, --pubkey <FILE>                   SSH public key
  -A, --cacert <FILE|DIR>               TLS CA certificate(s) to verify the peer against

 HTTP/SMTP:
  -H, --header <HEADER>                 Pass user-defined headers to the server

 Authentication:
  -m, --auth-mech <MECH>                Authentication mechanism (e.g. SCRAM-SHA-256)
  -u, --auth-user <USER>                The user to authenticate as
  -w, --auth-pwd <PASSWORD>             The user's password
  -C, --auth-cid <ID>                   SASL authentication identity
  -Z, --auth-zid <ID>                   SASL authorization identity

 Proxy:
  -P, --proxy <URL>                     Use a proxy, may be specified multiple times
      --proxy-header <HEADER>           Pass a header to the proxy

 Proxy TLS:
      --proxy-cert <FILE[:PASSWORD]>    Proxy TLS client certificate
      --proxy-key <FILE[:PASSWORD]>     Proxy TLS client private key
      --proxy-cacert <FILE|DIR>         Proxy TLS CA certificate(s) to verify the peer against

 Proxy Authentication:
      --proxy-auth-mech <MECH>          Authentication mechanism (e.g. SCRAM-SHA-256)
      --proxy-auth-user <USER>          The user to authenticate as
      --proxy-auth-pwd  <PASSWORD>      The user's password
      --proxy-auth-cid <ID>             SASL authentication identity
      --proxy-auth-zid <ID>             SASL authorization identity

 Error Handling:
  -F, --fail                            Return a non-zero exit code when the server returns an error
      --retry <RETRIES>                 Maximum number of retries, defaults to 1
      --retry-delay <SECONDS>           Wait time between retries
      --retry-max-time <SECONDS>        Retry only within this period
      --retry-error <ERROR>             Specify an error type on which to retry, may be specified multiple times

 Logging:
  -s, --silent                          Silent mode
  -v, --verbose                         Verbose mode
      --debug                           Set log level to debug
      --trace                           Set log level to trace

 Protocols:
      --http10
      --http11
      --http2
      --http3
      --tls12
      --tls13
      --ipv4
      --ipv6

Examples:
 net-cli https://rust-lang.org GET
 net-cli dns://8.8.8.8 rust-lang.org. AAAA IN
"#;

#[macro_export]
macro_rules! exit {
    ( $( $tt:tt )* ) => { {
		log::error!( $( $tt )* );
		std::process::exit(1);
	} };
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ProtocolVersion {
	Http10,
	Http11,
	Http20,
	Http30,
	Tls12,
	Tls13,
	IPv4,
	IPv6
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum InputFormat {

}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum OutputFormat {

}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ErrorType {
	Resolve(io::ErrorKind),
	Dns(net::dns::RCode),
	Connect(io::ErrorKind),
	Io(io::ErrorKind),
	Http(net::http::Status)
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Parameters {
	Tcp,
	Udp,
	Http { method: net::http::Method },
	Smtp { mail_from: Option<String>, rcpt_to: Vec<String> },
	Dns  { domain: Option<String>, qtype: net::dns::Type, qclass: net::dns::Class },
	Ldap { op: LdapOp }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum LdapOp {
	Bind,
	Search,
	Add,
	Modify,
	Del
}

#[derive(Clone, Debug, Default)]
struct Options {
	url:                   Option<Url>,
	params:                Option<Parameters>,
	log_silent:            bool,
	log_verbose:           bool,
	log_level:             log::Level,
	conn_peer:             IpSocketAddr,
	conn_socket:           Option<IpSocketAddr>,
	conn_interface:        Option<String>,
	conn_proxy:            Vec<Proxy>,
	conn_dns:              Vec<Dns>,
	conn_versions:         Vec<ProtocolVersion>,
	tcp_udp_conn_timeout:  Option<usize>,
	tcp_udp_ttl:           Option<usize>,
	tcp_udp_nodelay:       Option<bool>,
	tcp_udp_keep_alive:    Option<Duration>,
	tcp_udp_read_timeout:  Option<Duration>,
	tcp_udp_write_timeout: Option<Duration>,
	tcp_udp_recv_buffer:   Option<usize>,
	tcp_udp_send_buffer:   Option<usize>,
	tls:                    bool,
	tls_cert:               Option<Box<[u8]>>,
	tls_privkey:            Option<Box<[u8]>>,
	tls_pubkey:             Option<Box<[u8]>>,
	tls_cacert:             Option<Box<[u8]>>,
	auth_mech:              Option<AuthMech>,
	auth_user:              Option<String>,
	auth_pwd:               Option<String>,
	auth_cid:               Option<String>,
	auth_zid:               Option<String>,
	io_data:	            Option<Box<[u8]>>,
	io_input:               Option<Box<dyn io::Read>>,
	io_input_format:        Option<InputFormat>,
	io_output:              Option<Box<dyn io::Write>>,
	io_output_format:       Option<OutputFormat>,
	fail:                   bool,
	retry:                  usize,
	retry_max_time:         Option<Duration>,
	retry_delay:            Option<Duration>,
	retry_error:            Vec<ErrorType>,
	timeout:                Option<Duration>,
	headers:                Vec<String>,
}

impl Options {
	fn from_iter(iter: impl IntoIterator<Item = String>) -> Self {
		let args = iter.into_iter();
		let mut opts = Self::default();
		let mut param = 0;

		while let Some(arg) = args.next() {
			match arg {
				arg if !arg.starts_with('-') => {
					match (&mut opts.params, param) {
						(None, 0) => {
							let end = match arg.find("://") {
								Some(v) => v,
								None => exit!("no URL scheme specified")
							};

							opts.params = Some(match &arg[..end] {
								"tcp"            => Parameters::Tcp,
								"udp"            => Parameters::Udp,
								"http" | "https" => Parameters::Http { method: net::http::Method::Get },
								"smtp" | "smtps" => Parameters::Smtp { mail_from: None, rcpt_to: Vec::new() },
								"dns"  | "dot"   => Parameters::Dns  { domain: None, qtype: net::dns::Type::A, qclass: net::dns::Class::IN },
								"ldap" | "ldaps" => Parameters::Ldap { op: LdapOp::Bind },
								scheme           => exit!("invalid scheme: `{}`", scheme)
							});

							opts.url = Some(arg);
						},
						(Some(Parameters::Http { method }), 1) => match net::http::Method::from_str(&arg) {
							Ok(v)   => *method = v,
							Err(()) => exit!("invalid HTTP method: `{}`", arg)
						}
						(Some(Parameters::Smtp { mail_from, .. }), 1) => *mail_from = Some(arg),
						(Some(Parameters::Smtp { rcpt_to, .. }), _)   => rcpt_to.push(arg),
						(Some(Parameters::Dns  { domain, .. }), 1)    => domain = Some(arg),
						(Some(Parameters::Dns  { qtype, .. }), 2)     => qtype = arg,
						(Some(Parameters::Dns  { qclass, .. }), 3)    => qclass = arg,
						(Some(Parameters::Ldap { op }), 1)            => *op = match &arg {
							"BIND"   => LdapOp::Bind,
							"SEARCH" => LdapOp::Search,
							"ADD"    => LdapOp::Add,
							"MODIFY" => LdapOp::Modify,
							"DEL"    => LdapOp::Del,
							arg      => exit!("invalid LDAP operation: `{}`", arg),
						},
						param => exit!("unexpected request parameter: `{}`", arg)
					}

					param += 1;
				},
				"-V" | "--version" => {
					println!("{}", VERSION);
					std::process::exit(0);
				}
				"-h" | "--help" => {
					println!("{}", HELP);
					std::process::exit(0);
				}
				"-s" | "--silent"    => opts.log_silent = true,
				"-v" | "--verbose"   => opts.log_verbose = true,
				"-S" | "--socket"    => opts.conn_socket = match std::net::SocketAddr::from_str(arg) {
					Ok(v) => v,
					Err(_) => exit!("invalid value for -S/--socket")
				},
				"-I" | "--interface" => opts.conn_interface = Some(arg),
				"-D" | "--dns"       => opts.conn_dns.push(arg),
				"-H" | "--header"    => opts.headers.push(args.next()),
				"-c" | "--cert"      => {
					let path = args.next().unwrap();

					let data = match std::fs::read(&path) {
						Ok(v) => v,
						Err(e) => exit!("failed to read TLS client certificate from {}: {}", path, e)
					};

					opts.tls_cert = Some(data);
				}
				"-k" | "--privkey"   => {
					let path = args.next().unwrap();

					let data = match std::fs::read(&path) {
						Ok(v) => v,
						Err(e) => exit!("failed to read TLS client certificate from {}: {}", path, e)
					};

					opts.tls_cert = Some(data);
				}
				"-K" | "--pubkey"    => {
					let path = args.next().unwrap();

					let data = match std::fs::read(&path) {
						Ok(v) => v,
						Err(e) => exit!("failed to read TLS client certificate from {}: {}", path, e)
					};

					opts.tls_cert = Some(data);
				}
				"-A" | "--cacert"    => {
					let path = args.next().unwrap();

					let data = match std::fs::read(&path) {
						Ok(v) => v,
						Err(e) => exit!("failed to read TLS client certificate from {}: {}", path, e)
					};

					opts.tls_cert = Some(data);
				}
				"--retry"            => opts.retry = match arg.parse() {
					Ok(v) => v,
					Err(_) => exit!("invalid value for option --retry")
				},
				"--http10"           => opts.conn_versions.push(ProtocolVersion::Http10),
				"--http11"           => opts.conn_versions.push(ProtocolVersion::Http11),
				"--http20"           => opts.conn_versions.push(ProtocolVersion::Http20),
				"--http30"           => opts.conn_versions.push(ProtocolVersion::Http30),
				"--tls12"            => opts.conn_versions.push(ProtocolVersion::Tls12),
				"--tls13"            => opts.conn_versions.push(ProtocolVersion::Tls13),
				"--ipv4"             => opts.conn_versions.push(ProtocolVersion::IPv4),
				"--ipv6"             => opts.conn_versions.push(ProtocolVersion::IPv6),
				_ => exit!("invalid option: {}", arg)
			}
		}
	}
}

fn main() {
	let mut opts = Options::from_iter(std::env::args());

	match &opts.params {
		None => run_interactive().unwrap(),
		Some(Parameters::Http { .. }) => http::run(opts),
		_ => unimplemented!()
	}
}

fn run_interactive() -> io::Result<()> {
	let stdout  = std::io::stdout();
	let stdout  = stdout.lock();
	let stderr  = std::io::stderr();
	let stderr  = stderr.lock();
	let stdin   = std::io::stdin();
	let stdin   = stdin.lock();
	let mut buf = String::new();

	writeln!(&mut stdout, "{}", VERSION);

	loop {
		stdout.write_all(b"> ")?;
		stdout.flush()?;
		buf.clear();
		stdin.read_line(&mut buf)?;

		match buf.as_str() {
			"version"          => writeln!(&mut stdout, "{}", VERSION),
			"?" | "h" | "help" => writeln!(&mut stdout, "{}", HELP),
			cmd                => writeln!(&mut stdout, "Invalid command: {}", cmd),
		}?;
	}
}

