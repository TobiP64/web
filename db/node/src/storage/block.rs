use crate::storage::VirtAddr;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Block {
	pub ccn:  u128,
	pub ty:   BlockType
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum BlockType {
	Empty,
	Header,
	Index,
	Data
}

pub fn alloc(db: Uuid, table: Uuid) -> *mut () {
	unimplemented!()
}

pub fn free(db: Uuid, table: Uuid, addr: *mut ()) {
	unimplemented!()
}