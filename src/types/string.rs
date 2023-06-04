use crate::context::ASTContext;
use crate::types::bool::BoolType;
use crate::types::{BaseTypes, TypeBase};

use std::any::Any;
use std::ffi::CString;

extern crate llvm_sys;
use crate::types::llvm::*;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

#[derive(Debug, Clone)]
pub struct StringType {
    name: String,
    llmv_value: LLVMValueRef,
    length: *mut usize,
    llmv_value_pointer: Option<LLVMValueRef>,
    str_value: String,
}

impl TypeBase for StringType {
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        let value_as_string = match _value.downcast_ref::<String>() {
            Some(val) => val.to_string(),
            None => panic!("The input value must be a bool"),
        };
        let string: CString = CString::new(value_as_string.clone()).unwrap();
        unsafe {
            let value = LLVMConstStringInContext(
                _context.context,
                string.as_ptr(),
                string.as_bytes().len() as u32,
                0,
            );
            let mut len_value: usize = string.as_bytes().len();
            let ptr: *mut usize = (&mut len_value) as *mut usize;
            let buffer_ptr = LLVMBuildPointerCast(
                _context.builder,
                value,
                LLVMPointerType(LLVMInt8Type(), 0),
                c_str(_name.as_str()),
            );
            Box::new(StringType {
                name: _name,
                length: ptr,
                llmv_value: value,
                llmv_value_pointer: Some(buffer_ptr),
                str_value: value_as_string, // fix
            })
        }
    }
    fn assign(&self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        match _rhs.get_type() {
            BaseTypes::String => unsafe {
                let alloca = self.get_ptr();
                let name = LLVMGetValueName(self.get_value());
                let new_value = LLVMBuildLoad2(_ast_context.builder, int8_type(), alloca, name);
                LLVMBuildStore(_ast_context.builder, new_value, alloca);
            },
            _ => {
                unreachable!(
                    "Can't reassign variable {:?} that has type {:?} to type {:?}",
                    self.name,
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::String
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
    fn set_value(&mut self, _value: LLVMValueRef) {
        self.llmv_value = _value;
    }
    fn get_length(&self) -> *mut usize {
        self.length
    }
    fn get_ptr(&self) -> LLVMValueRef {
        match self.llmv_value_pointer {
            Some(v) => v,
            None => {
                unreachable!("No pointer for this value")
            }
        }
    }
    fn get_str(&self) -> String {
        self.str_value.clone()
    }

    fn add(&self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::String => match _ast_context.llvm_func_cache.get("sprintf") {
                Some(_sprintf_func) => unsafe {
                    // TODO: Use sprintf to concatenate two strings
                    // Remove extra quotes
                    let new_string =
                        format!("{}{}", self.get_str(), _rhs.get_str()).replace('\"', "");

                    let string = CString::new(new_string.clone()).unwrap();
                    let value = LLVMConstStringInContext(
                        _ast_context.context,
                        string.as_ptr(),
                        string.as_bytes().len() as u32,
                        0,
                    );
                    let mut len_value: usize = string.as_bytes().len();
                    let ptr: *mut usize = (&mut len_value) as *mut usize;
                    let buffer_ptr = LLVMBuildPointerCast(
                        _ast_context.builder,
                        value,
                        LLVMPointerType(LLVMInt8Type(), 0),
                        c_str("buffer_ptr"),
                    );
                    Box::new(StringType {
                        name: self.name.clone(),
                        length: ptr,
                        llmv_value: value,
                        llmv_value_pointer: Some(buffer_ptr),
                        str_value: new_string,
                    })
                },
                _ => {
                    unreachable!()
                }
            },
            _ => {
                unreachable!(
                    "Can't add type {:?} and type {:?}",
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }

    fn eqeq(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::String => {
                let value = self.get_str() == _rhs.get_str();
                return BoolType::new(Box::new(value), self.name.clone(), _context);
            }
            _ => {
                unreachable!(
                    "Can't compare == on dtype {:?} and type {:?}",
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }

    fn ne(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::String => {
                let value = self.get_str() != _rhs.get_str();
                return BoolType::new(Box::new(value), self.name.clone(), _context);
            }
            _ => {
                unreachable!(
                    "Can't compare != on type {:?} and type {:?}",
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }

    fn print(&self, ast_context: &mut ASTContext) {
        unsafe {
            // Set Value
            // create string vairables and then function
            // This is the Main Print Func
            let llvm_value_to_cstr = LLVMGetAsString(self.llmv_value, self.length);

            let value_is_str =
                LLVMBuildGlobalStringPtr(ast_context.builder, c_str!("%s\n"), c_str!(""));

            // Load Value from Value Index Ptr
            let val = LLVMBuildGlobalStringPtr(
                ast_context.builder,
                llvm_value_to_cstr,
                llvm_value_to_cstr,
            );

            let print_args = [value_is_str, val].as_mut_ptr();
            match ast_context.llvm_func_cache.get("printf") {
                Some(print_func) => {
                    LLVMBuildCall2(
                        ast_context.builder,
                        print_func.func_type,
                        print_func.function,
                        print_args,
                        2,
                        c_str!(""),
                    );
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}
