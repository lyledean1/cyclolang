extern crate llvm_sys;

use std::any::Any;
use std::ptr::null;
use llvm_sys::core::{LLVMArrayType2, LLVMConstArray2};
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use llvm_sys::prelude::*;
use crate::compiler::llvm::context::ASTContext;
use crate::compiler::types::BaseTypes::List;

#[derive(Debug, Clone)]
pub struct ListType {
    pub llvm_value: LLVMValueRef,
}

impl Base for ListType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Number
    }
}

impl TypeBase for ListType {
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase> {
        let value_as_expr_list = match _value.downcast_ref::<Box<Vec<Box<dyn TypeBase>>>>() {
            Some(val) => val,
            None => panic!("The input value is incorrect"),
        };
        let first_element = value_as_expr_list.get(0).unwrap();
        let mut elements = vec![];
        for x in value_as_expr_list.iter() {
            elements.push(x.get_value());
        }

        unsafe {
            let llvm_array_value = LLVMConstArray2(first_element.get_llvm_type(), elements.as_mut_ptr(), value_as_expr_list.len() as u64);
            Box::new(ListType {
                llvm_value: llvm_array_value,
            })
        }
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llvm_value
    }
}
impl Arithmetic for ListType {}

impl Comparison for ListType {}

impl Debug for ListType {}

impl Func for ListType {}
