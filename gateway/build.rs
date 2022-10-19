fn main() {
	capnpc::CompilerCommand::new()
		.file("gateway.capnp")
		.output_path("src/schema")
		.run()
		.expect("schema compiler command");
}
