use uuid::Uuid;

pub fn insert(table: Uuid) -> u32 {
	unimplemented!()
}

pub fn lookup_key(table: Uuid, index: Uuid, key: *mut [u8]) -> u32 {
	unimplemented!()
}

pub fn lookup_range(table: Uuid, index: Uuid, from: *mut [u8], to: *mut [u8]) -> u32 {
	unimplemented!()
}

pub fn lookup_all(table: Uuid, index: Uuid) -> u32 {
	unimplemented!()
}

pub fn cursor_fetch(cursor: u32) -> bool {
	unimplemented!()
}

pub fn cursor_fetch_concurrent(cursor: u32, function: ()) -> bool {
	unimplemented!()
}

pub fn cursor_close(cursor: u32) {
	unimplemented!()
}

pub fn cursor_delete(cursor: u32) {
	unimplemented!()
}

pub fn cursor_replace(cursor: u32) {
	unimplemented!()
}

pub fn cursor_read_column(cursor: u32, column: Uuid) -> Option<*mut [u8]> {
	unimplemented!()
}

pub fn cursor_read_next_column(cursor: u32) -> Option<(Uuid, *mut [u8])> {
	unimplemented!()
}

pub fn cursor_write_column(cursor: u32, column: Uuid, len: Option<usize>) -> Option<*mut u8> {
	unimplemented!()
}