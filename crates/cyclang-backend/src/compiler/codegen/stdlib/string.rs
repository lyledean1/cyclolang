use std::collections::HashMap;
use std::ffi::CString;
use llvm_sys::core::{LLVMFunctionType, LLVMGetNamedFunction, LLVMGetTypeByName2, LLVMPointerType, LLVMVoidTypeInContext};
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMContextRef, LLVMModuleRef};
use cyclang_parser::Type;
use crate::compiler::codegen::context::{LLVMFunction, LLVMFunctionCache};
use crate::compiler::codegen::int8_ptr_type;

pub unsafe fn load_string_helper_funcs(context: LLVMContextRef,
                                        module: LLVMModuleRef,
                                        llvm_func_cache: &mut LLVMFunctionCache, block: LLVMBasicBlockRef
) {
    let void_type: *mut llvm_sys::LLVMType = LLVMVoidTypeInContext(context);

    let string_struct_name =
        CString::new("struct.StringType").expect("CString::new failed");
    let string_type = LLVMGetTypeByName2(context, string_struct_name.as_ptr());
    let string_ptr_type = LLVMPointerType(string_type, 0);
    let string_init_function_name =
        CString::new("stringInit").expect("CString::new failed");
    let string_init_function =
        LLVMGetNamedFunction(module, string_init_function_name.as_ptr());

    // todo: load array correctly
    let mut string_init_args = [int8_ptr_type()];
    let string_init_func_type = LLVMFunctionType(
        string_ptr_type,
        string_init_args.as_mut_ptr(),
        string_init_args.len() as u32,
        0,
    );
    llvm_func_cache.set(
        "stringInit",
        LLVMFunction {
            function: string_init_function,
            func_type: string_init_func_type,
            block,
            entry_block: block,
            symbol_table: HashMap::new(),
            args: vec![int8_ptr_type()],
            return_type: Type::None,
        },
    );

    let string_add_function_name = CString::new("stringAdd").expect("CString::new failed");
    let string_add_function =
        LLVMGetNamedFunction(module, string_add_function_name.as_ptr());

    let mut string_add_args = [string_ptr_type, string_ptr_type];
    let string_add_func_type = LLVMFunctionType(
        void_type,
        string_add_args.as_mut_ptr(),
        string_add_args.len() as u32,
        0,
    );
    llvm_func_cache.set(
        "stringAdd",
        LLVMFunction {
            function: string_add_function,
            func_type: string_add_func_type,
            block,
            entry_block: block,
            symbol_table: HashMap::new(),
            args: vec![string_ptr_type, string_ptr_type],
            return_type: Type::None,
        },
    );


    let string_print_function_name = CString::new("stringPrint").expect("CString::new failed");
    let string_print_function =
        LLVMGetNamedFunction(module, string_print_function_name.as_ptr());

    let mut string_print_args = [string_ptr_type];
    let string_print_func_type = LLVMFunctionType(
        void_type,
        string_print_args.as_mut_ptr(),
        string_print_args.len() as u32,
        0,
    );
    llvm_func_cache.set(
        "stringPrint",
        LLVMFunction {
            function: string_print_function,
            func_type: string_print_func_type,
            block,
            entry_block: block,
            symbol_table: HashMap::new(),
            args: vec![string_ptr_type],
            return_type: Type::None,
        },
    );
}