## Crates

| Crate                             | Description
|:----------------------------------|:---
| cli                               | CLI for interacting with servers using various protocols
| common/async-executor             | Simple executor for futures
| common/debug-impl                 | Wrapper for types that do no implement the Debug trait
| common/dyn-error                  | Typless error wrapper
| common/otel-mrt                   | Minimal OpenTelemetry runtime
| common/protocols                  | Implementation of common Internet protocols
| common/quote-html                 | Proc macro for embedding HTML in Rust code
| common/serde-dyn-repr             | Typless representation of deserialized data
| common/stdout-log                 | Simple logger that logs to stdout
| db/console-backend                | Backend of the DB management web console
| db/console-frontend               | Frontend of the DB management web console
| db/driver                         | Client driver for connection to DB nodes
| db/grafana-datasource-backend     | Backend of the Grafana datasource
| db/grafana-datasource-frontend    | Frontend of the Grafana datasource
| db/node                           | Single instance of the DB
| router/api                        | Router for plugins to interact with the router
| router/api-controller             | Proc macro for defining routing rules in code
| router/console-backend            | Backend of the router management web console
| router/console-frontend           | Frontend of the router management web console
| router/db-bridge                  | Router plugin for storing configuration in the DB
| router/node                       | Single instance of the router

## Requirements

- Rust 1.69 nightly or higher
- LLVM