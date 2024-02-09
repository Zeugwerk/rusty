use csbindgen::Builder;

fn main() {
    csbindgen::Builder::default()
        .input_extern_file("../../../compiler/plc_driver/src/runner.rs")
        .csharp_dll_name("rustycs")
        .generate_csharp_file("../dotnet/NativeMethods.g.cs")
        .unwrap();
}
