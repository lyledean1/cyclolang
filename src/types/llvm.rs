use std::os::raw::c_ulonglong;

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

// Types

pub fn create_string_type(context: LLVMContextRef) -> LLVMTypeRef {
    unsafe {
        // Create an LLVM 8-bit integer type (i8) to represent a character
        let i8_type = LLVMInt8TypeInContext(context);

        // Create a pointer type to the i8 type to represent a string
        LLVMPointerType(i8_type, 0)
    }
}
/// Convert this integer to LLVM's representation of a constant
/// integer.
pub unsafe fn int8(val: c_ulonglong) -> LLVMValueRef {
    LLVMConstInt(LLVMInt8Type(), val, LLVM_FALSE)
}
/// Convert this integer to LLVM's representation of a constant
/// integer.
// TODO: this should be a machine word size rather than hard-coding 32-bits.
pub fn int32(val: c_ulonglong) -> LLVMValueRef {
    unsafe { LLVMConstInt(LLVMInt32Type(), val, LLVM_FALSE) }
}

pub fn int1_type() -> LLVMTypeRef {
    unsafe { LLVMInt1Type() }
}

pub fn int1_ptr_type() -> LLVMTypeRef {
    unsafe { LLVMPointerType(LLVMInt1Type(), 0) }
}

pub fn int8_type() -> LLVMTypeRef {
    unsafe { LLVMInt8Type() }
}

pub fn int32_type() -> LLVMTypeRef {
    unsafe { LLVMInt32Type() }
}

pub fn int32_ptr_type() -> LLVMTypeRef {
    unsafe { LLVMPointerType(LLVMInt32Type(), 0) }
}

pub fn int8_ptr_type() -> LLVMTypeRef {
    unsafe { LLVMPointerType(LLVMInt8Type(), 0) }
}

pub fn c_str(format_str: &str) -> *const i8 {
    format_str.as_ptr() as *const i8
}

pub fn c_str_with_type(name: &str, type_name: &str) -> *const i8 {
    let type_name_str = name.to_owned() + type_name;
    c_str(&type_name_str)
}

pub fn var_type_str(name: String, type_name: String) -> String {
    name + "_" + &type_name
}

pub fn bool_type(context: LLVMContextRef, boolean: bool) -> LLVMValueRef {
    unsafe {
        let bool_type = LLVMInt1TypeInContext(context);

        // Create a LLVM value for the bool
        // Return the LLVMValueRef for the bool
        LLVMConstInt(bool_type, boolean as u64, 0)
    }
}

pub fn get_i32_value(value: LLVMValueRef) -> i32 {
    let zext_value: c_ulonglong = unsafe { LLVMConstIntGetZExtValue(value) };
    zext_value as i32
}
