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

pub enum T {
	// geo objects, arrays, sets
	Nearest,
	Farthest,
	Distance,
	
	// CRUD
	Insert,
	Update,
	Replace,
	Delete
}

pub type Context  = ();
pub type Utf8Str  = (u32, u32);
pub type Database = u32;
pub type Table    = u32;
pub type Index    = u32;
pub type Stream   = u32;
pub type Object   = u32;
pub type Function = u32;
pub type DateTime = i128;

pub fn db_list(ctx: &mut Context) {

}

pub fn db_select(ctx: &mut Context, name: Utf8Str) {

}

pub fn db_create(ctx: &mut Context, name: Utf8Str) {

}

pub fn db_drop(ctx: &mut Context, db: Database) {

}

pub fn db_rebalance(ctx: &mut Context, db: Database) {

}

pub fn db_reconfigure(ctx: &mut Context, db: Database) {

}

pub fn db_status(ctx: &mut Context, db: Database) {

}

pub fn db_wait(ctx: &mut Context, db: Database) {

}

pub fn db_sync(ctx: &mut Context, db: Database) {

}

pub fn table_list(ctx: &mut Context, db: Database) {

}

pub fn table_select(ctx: &mut Context, db: Database, name: Utf8Str) {

}

pub fn table_create(ctx: &mut Context, db: Database, name: Utf8Str) {

}

pub fn table_drop(ctx: &mut Context, table: Table) {

}

pub fn table_rebalance(ctx: &mut Context, table: Table) {

}

pub fn table_reconfigure(ctx: &mut Context, table: Table) {

}

pub fn table_status(ctx: &mut Context, table: Table) {

}

pub fn table_wait(ctx: &mut Context, table: Table) {

}

pub fn table_sync(ctx: &mut Context, table: Table) {

}

pub fn table_set_write_hook(ctx: &mut Context, table: Table, function: Function) {

}

pub fn table_get_write_hook(ctx: &mut Context, table: Table) -> Function {
	unimplemented!()
}

pub fn index_list(ctx: &mut Context, table: Table) {

}

pub fn index_select(ctx: &mut Context, table: Table, name: Utf8Str) {

}

pub fn index_create(ctx: &mut Context, table: Table, name: Utf8Str) {

}

pub fn index_drop(ctx: &mut Context, index: Index) {

}

pub fn index_status(ctx: &mut Context, index: Index) {

}

pub fn index_wait(ctx: &mut Context, index: Index) {

}

pub fn index_get(ctx: &mut Context, index: Index, key: Object) -> Object {
	unimplemented!()
}

pub fn index_get_all(ctx: &mut Context, index: Index, keys: (u32, u32)) -> Stream {
	unimplemented!()
}

pub fn index_get_range(ctx: &mut Context, index: Index, from: Object, to: Object) -> Stream {
	unimplemented!()
}

pub fn string_trim(ctx: &mut Context, s: Utf8Str, pat: Utf8Str) -> Utf8Str {
	unimplemented!()
}

pub fn string_trim_start(ctx: &mut Context, s: Utf8Str, pat: Utf8Str) -> Utf8Str {
	unimplemented!()
}

pub fn string_trim_end(ctx: &mut Context, s: Utf8Str, pat: Utf8Str) -> Utf8Str {
	unimplemented!()
}

pub fn string_strip(ctx: &mut Context, s: Utf8Str, pat: Utf8Str) -> Utf8Str {
	unimplemented!()
}

pub fn string_strip_start(ctx: &mut Context, s: Utf8Str, pat: Utf8Str) -> Utf8Str {
	unimplemented!()
}

pub fn string_strip_end(ctx: &mut Context, s: Utf8Str, pat: Utf8Str) -> Utf8Str {
	unimplemented!()
}

pub fn string_contains(ctx: &mut Context, s: Utf8Str, pat: Utf8Str) -> bool {
	unimplemented!()
}

pub fn string_matches(ctx: &mut Context, s: Utf8Str) -> bool {
	unimplemented!()
}

pub fn string_starts_with(ctx: &mut Context, s: Utf8Str, pat: Utf8Str) -> bool {
	unimplemented!()
}

pub fn string_to_lowercase(ctx: &mut Context, s: Utf8Str) {
	unimplemented!()
}

pub fn string_to_uppercase(ctx: &mut Context, s: Utf8Str) {
	unimplemented!()
}

pub fn collection_is_empty(ctx: &mut Context, coll: Object) -> bool {
	unimplemented!()
}

pub fn collection_length(ctx: &mut Context, coll: Object) -> u32 {
	unimplemented!()
}

pub fn seq_append(ctx: &mut Context, seq: Object, val: Object) {

}

pub fn seq_prepend(ctx: &mut Context, seq: Object, val: Object) {

}

pub fn seq_splice(ctx: &mut Context, seq: Object, index: u32, other: Object) {

}

pub fn seq_insert(ctx: &mut Context, seq: Object, index: u32, val: Object) {

}

pub fn seq_replace(ctx: &mut Context, seq: Object, index: u32, val: Object) {

}

pub fn seq_delete(ctx: &mut Context, seq: Object, index: u32) {

}

pub fn seq_intersects(ctx: &mut Context, seq: Object, other: Object) -> bool {
	unimplemented!()
}

pub fn seq_union(ctx: &mut Context, seq: Object, other: Object) {

}

pub fn seq_intersection(ctx: &mut Context, seq: Object, other: Object) {

}

pub fn seq_difference(ctx: &mut Context, seq: Object, other: Object) {

}

pub fn set_insert(ctx: &mut Context, set: Object, val: Object) {

}

pub fn set_replace(ctx: &mut Context, set: Object, val: Object, new: Object) {

}

pub fn set_delete(ctx: &mut Context, set: Object, val: Object) {

}

pub fn set_intersects(ctx: &mut Context, set: Object, other: Object) -> bool {
	unimplemented!()
}

pub fn set_union(ctx: &mut Context, set: Object, other: Object) {

}

pub fn set_intersection(ctx: &mut Context, set: Object, other: Object) {

}

pub fn set_difference(ctx: &mut Context, set: Object, other: Object) {

}

pub fn map_get(ctx: &mut Context, map: Object, key: Utf8Str) -> Object {
	unimplemented!()
}

pub fn map_insert(ctx: &mut Context, map: Object, key: Utf8Str, val: Object) {
	unimplemented!()
}

pub fn map_delete(ctx: &mut Context, map: Object, key: Utf8Str) {
	unimplemented!()
}

pub fn map_union(ctx: &mut Context, map: Object, other: Object) {

}

pub fn map_has_fields(ctx: &mut Context, map: Object, fields: Object) -> bool {
	unimplemented!()
}

pub fn map_with_fields(ctx: &mut Context, map: Object, fields: Object) {

}

pub fn map_without_fields(ctx: &mut Context, map: Object, fields: Object) {

}

pub fn map_keys(ctx: &mut Context, map: Object) -> Stream {
	unimplemented!()
}

pub fn map_values(ctx: &mut Context, map: Object) -> Stream {
	unimplemented!()
}

pub fn stream_get(ctx: &mut Context) -> Stream {
	unimplemented!()
}

pub fn stream_fetch(ctx: &mut Context, stream: Stream) -> Object {
	unimplemented!()
}

// stream transformation

pub fn stream_filter(ctx: &mut Context, stream: Stream, function: Function) {
	unimplemented!()
}

pub fn stream_map(ctx: &mut Context, stream: Stream, function: Function) {
	unimplemented!()
}

pub fn stream_concat(ctx: &mut Context, stream: Stream) {
	unimplemented!()
}

pub fn stream_order_by(ctx: &mut Context, stream: Stream, function: Function) {
	unimplemented!()
}

pub fn stream_nth(ctx: &mut Context, stream: Stream, index: u32) -> Object {
	unimplemented!()
}

pub fn stream_skip(ctx: &mut Context, stream: Stream, skip: u32) -> Object {
	unimplemented!()
}

pub fn stream_limit(ctx: &mut Context, stream: Stream, limit: u32) -> Object {
	unimplemented!()
}

pub fn stream_slice(ctx: &mut Context, stream: Stream, from: u32, to: u32) -> Object {
	unimplemented!()
}

pub fn stream_sample(ctx: &mut Context, stream: Stream, number: u32) -> Object {
	unimplemented!()
}

pub fn stream_union(ctx: &mut Context, stream: Stream, other: Stream) {
	unimplemented!()
}

pub fn stream_intersection(ctx: &mut Context, stream: Stream, other: Stream) {
	unimplemented!()
}

pub fn stream_difference(ctx: &mut Context, stream: Stream, other: Stream) {
	unimplemented!()
}

// stream aggregation

pub fn stream_to_array(ctx: &mut Context, stream: Stream) -> u32 {
	unimplemented!()
}

pub fn stream_group(ctx: &mut Context, stream: Stream, function: Function) {
	unimplemented!()
}

pub fn stream_fold(ctx: &mut Context, stream: Stream, function: Function) {
	unimplemented!()
}

pub fn stream_distinct(ctx: &mut Context, stream: Stream) {
	unimplemented!()
}

pub fn stream_count(ctx: &mut Context, stream: Stream) -> u64 {
	unimplemented!()
}

pub fn stream_sum(ctx: &mut Context, stream: Stream) -> f64 {
	unimplemented!()
}

pub fn stream_avg(ctx: &mut Context, stream: Stream) -> f64 {
	unimplemented!()
}

pub fn stream_min(ctx: &mut Context, stream: Stream) -> f64 {
	unimplemented!()
}

pub fn stream_max(ctx: &mut Context, stream: Stream) -> f64 {
	unimplemented!()
}

pub fn stream_percentile(ctx: &mut Context, stream: Stream, percentile: f64) -> f64 {
	unimplemented!()
}

pub fn stream_is_empty(ctx: &mut Context, stream: Stream) -> bool {
	unimplemented!()
}

pub fn stream_contains_one(ctx: &mut Context, stream: Stream, value: u32) -> bool {
	unimplemented!()
}

pub fn stream_contains(ctx: &mut Context, stream: Stream, other: Stream) -> bool {
	unimplemented!()
}

pub fn stream_intersects(ctx: &mut Context, stream: Stream, other: Stream) -> bool {
	unimplemented!()
}

pub fn datetime_now(ctx: &mut Context) -> DateTime {
	unimplemented!()
}

pub fn uuid(ctx: &mut Context) -> u128 {
	unimplemented!()
}

pub fn random(ctx: &mut Context, from: f64, to: f64) -> f64 {
	unimplemented!()
}

pub fn abort(ctx: &mut Context, msg: Utf8Str) -> ! {
	unimplemented!()
}