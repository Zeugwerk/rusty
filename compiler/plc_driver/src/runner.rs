use std::{ffi::{c_char, CStr, CString}, fs, ptr::{self}};

use crate::{pipelines::ParsedProject, CompileOptions};

use plc::codegen::{CodegenContext, GeneratedModule};
use plc_diagnostics::diagnostician::Diagnostician;
use plc_index::GlobalContext;
use project::project::Project;
use source_code::Compilable;

#[allow(dead_code)]
#[repr(C)]
pub struct MainType {
    a: [usize; 1000],
}

impl Default for MainType {
    fn default() -> Self {
        MainType { a: [0; 1000] }
    }
}

///
/// Compiles and runs the given sources
/// Sources must be `Compilable`, default implementations include `String` and `&str`
/// An implementation is also provided for `Vec<SourceContainer>`
///
pub fn compile<T: Compilable>(context: &CodegenContext, source: T) -> GeneratedModule<'_> {
    let source = source.containers();
    let project = Project::new("TestProject".to_string()).with_sources(source);
    let ctxt = GlobalContext::new().with_source(project.get_sources(), None).unwrap();
    let mut diagnostician = Diagnostician::null_diagnostician();
    let parsed_project = ParsedProject::parse(&ctxt, &project, &mut diagnostician).unwrap();
    let indexed_project = parsed_project.index(ctxt.provider());
    let annotated_project = indexed_project.annotate(ctxt.provider());
    let compile_options = CompileOptions {
        optimization: plc::OptimizationLevel::None,
        debug_level: plc::DebugLevel::None,
        ..Default::default()
    };

    annotated_project.generate_single_module(context, &compile_options).unwrap().unwrap()
}

/// # Safety
///
/// This function is marked as unsafe because it dereferences a raw pointer.
/// Callers must ensure that the pointer is valid and properly aligned.
/// Dereferencing an invalid or unaligned pointer can lead to undefined behavior.
///
/// Parses the given sources
/// Sources must be `Compilable`, default implementations include `String` and `&str`
/// An implementation is also provided for `Vec<SourceContainer>`
#[no_mangle]
pub unsafe extern "C" fn parse(file_path: *const c_char) -> *const std::os::raw::c_char {

    // Convert the C string to Rust string
    let c_str = unsafe {
        assert!(!file_path.is_null());
        CStr::from_ptr(file_path)
    };
    let path_str = c_str.to_str().expect("Invalid UTF-8 in file path");

    // Read the file content
    let file_content = match fs::read_to_string(path_str) {
        Ok(content) => content,
        Err(_) => return std::ptr::null(), // Return null pointer on error
    };

    let source = file_content.containers();
    let project = Project::new("TestProject".to_string()).with_sources(source);
    let ctxt = GlobalContext::new().with_source(project.get_sources(), None).unwrap();
    let mut diagnostician = Diagnostician::null_diagnostician();
    let parsed_project = ParsedProject::parse(&ctxt, &project, &mut diagnostician).unwrap();

    let json = serde_json::to_string(&parsed_project);

    // Extract the string from the result
    let json_string = match json {
        Ok(json) => json,
        Err(_err) => {
            panic!("Serialization failed");
        }
    };

    // Convert the JSON string to a CString
    let c_string = match CString::new(json_string) {
        Ok(c_string) => c_string,
        Err(_err) => {
            return ptr::null();
        }
    };

    let raw_ptr = c_string.into_raw();
    raw_ptr as *const c_char
}


///
/// A Convenience method to compile and then run the given source
///
pub fn compile_and_run<T, U, S: Compilable>(source: S, params: &mut T) -> U {
    let context: CodegenContext = CodegenContext::create();
    let module = compile(&context, source);
    module.print_to_stderr();
    module.run::<T, U>("main", params)
}

///
/// A Convenience method to compile and then run the given source
/// without external parameters
///
pub fn compile_and_run_no_params<U, S: Compilable>(source: S) -> U {
    let context: CodegenContext = CodegenContext::create();
    let module = compile(&context, source);
    module.print_to_stderr();
    module.run_no_param::<U>("main")
}
