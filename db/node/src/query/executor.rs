pub struct Context {
	engine:  wasmer::UniversalEngine,
	store:   wasmer::Store,
	imports: wasmer::ImportObject,
}

pub fn init() -> Context {
	let compiler = wasmer_compiler_llvm::LLVM::new();
	let engine   = wasmer::Universal::new(compiler).engine();
	let store    = wasmer::Store::new(&engine);
	
	let mut namespace = wasmer::Exports::new();
	namespace.insert("insert",       wasmer::Function::new_native(&store, super::rawops::insert));
	namespace.insert("lookup_key",   wasmer::Function::new_native(&store, super::rawops::lookup_key));
	namespace.insert("lookup_range", wasmer::Function::new_native(&store, super::rawops::lookup_range));
	namespace.insert("lookup_all",   wasmer::Function::new_native(&store, super::rawops::lookup_all));
	
	let mut imports = wasmer::ImportObject::new();
	imports.register("dbops", namespace);
	
	Context {
		engine,
		store,
		imports
	}
}

pub async fn execute(ctx: &Context, query: super::Request) {
	let module   = wasmer::Module::new(&ctx.store, &query.query).unwrap();
	let instance = wasmer::Instance::new(&module, &ctx.imports).unwrap();
	let query    = instance.exports.get_function("query").unwrap();
	query.call(&[]).unwrap();
}