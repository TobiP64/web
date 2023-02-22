use crate::transaction::allocator::VERSION_SIZE;

pub const VERSION_BASE: u64 = 0x0010_0000_0000_0000; // user space ends at 0x0000_8000_0000_0000
pub const VERSION_SIZE: u64 = 0x0010_0000_0000_0000; // 4 PB
pub const VERSION_TEMP: u64 = 0x0001_0000_0000_0000; // 256 TB
pub const VERSIONS_MAX: u64 = 0x1000;

pub mod snapshot;
pub mod version;
pub mod block;
pub mod row;
pub mod column;
pub mod btree;

type VirtAddr = u64;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Header {
	pub identifier:      u64,
	pub version:         u16,
	/// 2^n bytes (min = 12, max = 255), e.g. 12 = 4096, 13 = 8192, 14 = 16384
	pub block_size:      u8,
	pub _pad0:           u8,
	pub _pad1:           u32,
	/// always Header
	pub ty:              block::BlockType,
	pub tables_root:     VirtAddr,
	pub blocks_root:     VirtAddr,
}

