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

pub const fn uuid(s: &[u8]) -> Uuid {
    Uuid::from_u128(xxhash_rust::const_xxh3::xxh3_128(s))
}

pub const SYS_CLUSTER_CONFIG:  Uuid = uuid(b"sys.cluster_config");
pub const SYS_CLUSTER_STATUS:  Uuid = uuid(b"sys.cluster_status");
pub const SYS_NODE_CONFIG:     Uuid = uuid(b"sys.node_config");
pub const SYS_NODE_STATUS:     Uuid = uuid(b"sys.node_status");
pub const SYS_TABLE_CONFIG:    Uuid = uuid(b"sys.table_config");
pub const SYS_TABLE_CONFIG_ID: Uuid = uuid(b"sys.table_config.id");
pub const SYS_TABLE_CONFIG_NAME: Uuid = uuid(b"sys.table_config.name");
pub const SYS_TABLE_CONFIG_NODES: Uuid = uuid(b"sys.table_config.nodes");
pub const SYS_TABLE_CONFIG_INDEXES: Uuid = uuid(b"sys.table_config.indexes");
pub const SYS_TABLE_CONFIG_ATTRIBUTES: Uuid = uuid(b"sys.table_config.attributes");
pub const SYS_TABLE_CONFIG_WRITE_HOOK: Uuid = uuid(b"sys.table_config.write_hook");
pub const SYS_TABLE_STATUS:    Uuid = uuid(b"sys.table_status");
pub const SYS_QUERY_CONFIG:    Uuid = uuid(b"sys.query_config");
pub const SYS_LOGS:            Uuid = uuid(b"sys.logs");
pub const SYS_JOBS:            Uuid = uuid(b"sys.jobs");
pub const SYS_CONTEXT:         Uuid = uuid(b"sys.context");

pub struct Table {
    pub id:          Uuid,
    pub name:        String,
    pub ranges:      Vec<Range>,
    pub attributes:  Vec<Attribute>,
    pub indexes:     Vec<Index>
}

pub struct Range {
    pub range_start: Uuid,
    pub range_end:   Uuid,
}

pub struct Attribute {
    pub id:   Uuid,
    pub idx:  u32,
    pub name: Option<String>,
    pub ty:   AttributeType,
}

pub enum AttributeType {
    Map(Vec<Attribute>),
    Seq(Vec<Attribute>),
    Val(AttributeTypeValue)
}

pub struct Index {
    pub id:         Uuid,
    pub name:       Option<String>,
    pub primary:    bool,
    pub attributes: Vec<(Uuid, IndexAttributeType)>
}

pub enum IndexAttributeType {
    // index opaque attribute
    Simple,
    // index array values, meaning each row has multiple keys
    Multi,
    // index inner maps
    SubIndex,
    // index array or string/binary values
    RadixTree,
    Geospatial,
    Expr(Vec<u8>)
}

pub enum AttributeTypeValue {
    Int16,
    Int32,
    Int64,
    Int128,
    Float16,
    Float32,
    Float64,
    Float128,
    Null,
    Bool,
    String,
    Bytes,
    Map,
    Array,
    DateTime,
    Procedure,
    Type,
    MinVal,
    MaxVal,
    Point2,
    Point3,
    Points,
    Polyline,
    Polygon,
    Polyhedron,
}