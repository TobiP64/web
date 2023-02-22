
pub fn lookup<K, V>(base: *mut (), root: VirtAddr, node_size: usize, key: K) -> *mut V {
	unimplemented!()
}

pub fn lookup_range<K, V>(base: *mut (), root: VirtAddr, node_size: usize, from: K, to: K) -> RangeIter<K, V> {
	unimplemented!()
}

pub fn iter<K, V>(base: *mut (), root: VirtAddr, node_size: usize) -> Iter<K, V> {
	unimplemented!()
}

pub struct RangeIter<K, V> {

}