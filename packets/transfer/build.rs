use tq_wasm_builder::WasmBuilder;

fn main() {
    WasmBuilder::selector()
        .with_current_project()
        .enable_feature("std")
        .build();
}
