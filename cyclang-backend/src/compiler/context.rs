use crate::compiler::codegen::builder::LLVMCodegenBuilder;
use crate::compiler::codegen::context::LLVMFunction;
use crate::compiler::codegen::{
    cstr_from_string, int1_type, int32_ptr_type, int32_type, int64_ptr_type, int64_type,
};
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::func::FuncType;
use crate::compiler::types::list::ListType;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::num64::NumberType64;
use crate::compiler::types::return_type::ReturnType;
use crate::compiler::types::string::StringType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::TypeBase;
use crate::compiler::visitor::Visitor;
use crate::compiler::CompileOptions;
use crate::compiler::Expression;
use anyhow::anyhow;
use anyhow::Result;
use libc::c_ulonglong;
use llvm_sys::core::{
    LLVMBuildPointerCast, LLVMConstStringInContext, LLVMInt8Type, LLVMPointerType,
};
use std::collections::HashMap;
use std::ffi::CString;

pub struct ASTContext {
    pub var_cache: VariableCache,
    pub func_cache: VariableCache,
    pub depth: i32,
}

impl ASTContext {
    pub fn get_depth(&self) -> i32 {
        self.depth
    }
    pub fn incr(&mut self) {
        self.depth += 1;
    }
    pub fn decr(&mut self) {
        self.depth -= 1;
    }
}

#[derive(Clone)]
struct Container {
    pub trait_object: Box<dyn TypeBase>,
}
pub struct VariableCache {
    map: HashMap<String, Container>,
    local: HashMap<i32, Vec<String>>,
}

impl Default for VariableCache {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableCache {
    pub fn new() -> Self {
        VariableCache {
            map: HashMap::new(),
            local: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, trait_object: Box<dyn TypeBase>, depth: i32) {
        let mut locals: HashMap<i32, bool> = HashMap::new();
        locals.insert(depth, true);
        self.map.insert(key.to_string(), Container { trait_object });
        match self.local.get(&depth) {
            Some(val) => {
                let mut val_clone = val.clone();
                val_clone.push(key.to_string());
                self.local.insert(depth, val_clone);
            }
            None => {
                self.local.insert(depth, vec![key.to_string()]);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<Box<dyn TypeBase>> {
        match self.map.get(key) {
            Some(v) => Some(dyn_clone::clone_box(&*v.trait_object)),
            None => None,
        }
    }

    #[allow(dead_code)]
    fn del(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn del_locals(&mut self, depth: i32) {
        if let Some(v) = self.local.get(&depth) {
            for local in v.iter() {
                self.map.remove(&local.to_string());
            }
            self.local.remove(&depth);
        }
    }
}

impl ASTContext {
    pub fn init(compile_options: Option<CompileOptions>) -> anyhow::Result<ASTContext> {
        let var_cache = VariableCache::new();
        let func_cache = VariableCache::new();
        //TODO: remove
        Ok(ASTContext {
            var_cache,
            func_cache,
            depth: 0,
        })
    }

    pub fn match_ast(&mut self, input: Expression, visitor: &mut Box<dyn Visitor<Box<dyn TypeBase>>>, codegen: &mut LLVMCodegenBuilder) -> Result<Box<dyn TypeBase>> {
        match input {
            Expression::Number(_) => visitor.visit_number(&input, codegen),
            Expression::Number64(_) => visitor.visit_number(&input, codegen),
            Expression::String(_) => visitor.visit_string(&input, codegen),
            Expression::Bool(_) => visitor.visit_bool(&input, codegen),
            Expression::Variable(_) => visitor.visit_variable(&input, codegen, &self.var_cache),
            // Expression::List(_) => visitor.visit_list(&input, codegen, self),
            Expression::ListIndex(v, i) => {
                let name = cstr_from_string("access_array").as_ptr();
                let val = self.match_ast(*v, visitor, codegen)?;
                let index = self.match_ast(*i, visitor, codegen)?;
                let zero_index = codegen.const_int(int64_type(), 0, 0);
                let build_load_index =
                    codegen.build_load(index.get_ptr().unwrap(), index.get_llvm_type(), "example");
                let mut indices = [zero_index, build_load_index];
                let val = codegen.build_gep(
                    val.get_llvm_type(),
                    val.get_ptr().unwrap(),
                    indices.as_mut_ptr(),
                    2_u32,
                    name,
                );
                Ok(Box::new(NumberType {
                    llvm_value: val,
                    llvm_value_pointer: Some(val),
                    name: "".to_string(),
                }))
            }
            Expression::ListAssign(var, i, rhs) => match self.var_cache.get(&var) {
                Some(val) => {
                    let name = cstr_from_string("access_array").as_ptr();
                    let lhs: Box<dyn TypeBase> = self.match_ast(*rhs, visitor, codegen)?;
                    let index = self.match_ast(*i, visitor, codegen)?;
                    let zero_index = codegen.const_int(int64_type(), 0, 0);
                    let build_load_index =
                        codegen.build_load(index.get_ptr().unwrap(), index.get_llvm_type(), "example");
                    let mut indices = [zero_index, build_load_index];
                    let element_ptr = codegen.build_gep(
                        val.get_llvm_type(),
                        val.get_ptr().unwrap(),
                        indices.as_mut_ptr(),
                        2_u32,
                        name,
                    );
                    codegen.build_store(lhs.get_value(), element_ptr);
                    Ok(val)
                }
                _ => {
                    unreachable!("can't assign as var doesn't exist")
                }
            },
            Expression::Nil => {
                unimplemented!()
            }
            Expression::Binary(lhs, op, rhs) => {
                let lhs = self.match_ast(*lhs, visitor, codegen)?;
                let rhs = self.match_ast(*rhs, visitor, codegen)?;
                match op.as_str() {
                    "+" => {
                        codegen.arithmetic(lhs, rhs, op)
                    }
                    "-" => {
                        codegen.arithmetic(lhs, rhs, op)
                    }
                    "/" => {
                        codegen.arithmetic(lhs, rhs, op)
                    }
                    "*" => {
                        codegen.arithmetic(lhs, rhs, op)
                    }
                    "^" => Err(anyhow!("^ is not implemented yet")),
                    "==" => {
                        codegen.cmp(lhs, rhs, op)
                    }
                    "!=" => {
                        codegen.cmp(lhs, rhs, op)
                    }
                    "<" => {
                        codegen.cmp(lhs, rhs, op)
                    }
                    "<=" => {
                        codegen.cmp(lhs, rhs, op)
                    }
                    ">" => {
                        codegen.cmp(lhs, rhs, op)
                    }
                    ">=" => {
                        codegen.cmp(lhs, rhs, op)
                    }
                    _ => {
                        Err(anyhow!("Operator: {} not implement", op.clone()))
                    }
                }
            },
            Expression::Grouping(_input) => self.match_ast(*_input,visitor, codegen),
            Expression::LetStmt(var, _, lhs) => {
                match self.var_cache.get(&var) {
                    Some(mut val) => {
                        // Check Variables are the same Type
                        // Then Update the value of the old variable
                        // reassign variable

                        // Assign a temp variable to the stack
                        let lhs: Box<dyn TypeBase> = self.match_ast(*lhs,visitor, codegen)?;
                        // Assign this new value
                        val.assign(codegen, lhs)?;
                        Ok(val)
                    }
                    _ => {
                        let lhs = self.match_ast( *lhs,visitor, codegen)?;
                        self.var_cache.set(&var.clone(), lhs.clone(), self.depth);
                        Ok(lhs)
                    }
                }
            }
            Expression::BlockStmt(exprs) => {
                // Set Variable Depth
                // Each Block Stmt, Incr and Decr
                // Clearing all the "Local" Variables That Have Been Assigned
                self.incr();
                let mut val: Box<dyn TypeBase> = Box::new(VoidType {});
                for expr in exprs {
                    val = self.match_ast(expr, visitor, codegen)?;
                }
                // Delete Variables
                self.var_cache.del_locals(self.get_depth());
                self.decr();
                Ok(val)
            }
            Expression::CallStmt(name, args) => match self.func_cache.get(&name) {
                Some(val) => {
                    let call_val = val.call(self, args, visitor, codegen)?;
                    self.var_cache
                        .set(name.as_str(), call_val.clone(), self.depth);
                    Ok(call_val)
                }
                _ => {
                    Err(anyhow!("call does not exist for function {:?}", name))
                }
            },
            Expression::FuncStmt(name, args, _return_type, body) => {
                let llvm_func = LLVMFunction::new(
                    self,
                    name.clone(),
                    args.clone(),
                    _return_type.clone(),
                    *body.clone(),
                    codegen.current_function.block,
                    visitor,
                    codegen,
                )?;

                let func = FuncType {
                    llvm_type: llvm_func.func_type,
                    llvm_func: llvm_func.function,
                    return_type: _return_type,
                };
                // Set Func as a variable
                self.func_cache
                    .set(&name, Box::new(func.clone()), self.depth);
                Ok(Box::new(func))
            },
            Expression::FuncArg(arg_name, arg_type) => {
                Err(anyhow!("this should be unreachable code, for Expression::FuncArg arg_name:{} arg_type:{:?}", arg_name, arg_type))
            }
            Expression::IfStmt(condition, if_stmt, else_stmt) => {
                //TODO: fix this so its an associated function
                LLVMCodegenBuilder::new_if_stmt(self, *condition, *if_stmt, *else_stmt,visitor, codegen)
            }
            Expression::WhileStmt(condition, while_block_stmt) => {
                //TODO: fix this so its an associated function
                LLVMCodegenBuilder::new_while_stmt(self, *condition, *while_block_stmt,visitor, codegen)
            }
            Expression::ForStmt(var_name, init, length, increment, for_block_expr) => {
                //TODO: fix this so its an associated function
                LLVMCodegenBuilder::new_for_loop(self, var_name, init, length, increment, *for_block_expr,visitor, codegen)
            }
            Expression::Print(input) => {
                let expression_value = self.match_ast(*input, visitor, codegen)?;
                expression_value.print(self, codegen)?;
                Ok(expression_value)
            }
            Expression::ReturnStmt(input) => {
                let expression_value = self.match_ast(*input,visitor, codegen)?;
                codegen.build_ret(expression_value.get_value());
                Ok(Box::new(ReturnType {}))
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

pub struct LLVMCodegenVisitor {
    // codegen: LLVMCodegenBuilder,
    // var_cache: VariableCache,
}

impl Visitor<Box<dyn TypeBase>> for LLVMCodegenVisitor {
    fn visit_number(
        &mut self,
        left: &Expression,
        codegen: &LLVMCodegenBuilder,
    ) -> Result<Box<dyn TypeBase>> {
        match left {
            Expression::Number(val) => {
                let name = "num32";
                let c_val = *val as c_ulonglong;
                let value = codegen.const_int(int32_type(), c_val, 0);
                let ptr = codegen.build_alloca_store(value, int32_ptr_type(), name);
                Ok(Box::new(NumberType {
                    name: name.to_string(),
                    llvm_value: value,
                    llvm_value_pointer: Some(ptr),
                }))
            }
            Expression::Number64(val) => {
                let name = "num64";
                let c_val = *val as c_ulonglong;
                let value = codegen.const_int(int64_type(), c_val, 0);
                let ptr = codegen.build_alloca_store(value, int64_ptr_type(), name);
                Ok(Box::new(NumberType64 {
                    name: name.to_string(),
                    llvm_value: value,
                    llvm_value_pointer: Some(ptr),
                }))
            }
            _ => Err(anyhow!("type is not a number (i32,i64)")),
        }
    }

    fn visit_string(
        &mut self,
        left: &Expression,
        codegen: &LLVMCodegenBuilder,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::String(val) = left {
            let name = "str_val";
            let string: CString = CString::new(val.clone()).unwrap();
            unsafe {
                let value = LLVMConstStringInContext(
                    codegen.context,
                    string.as_ptr(),
                    string.as_bytes().len() as u32,
                    0,
                );
                let mut len_value: usize = string.as_bytes().len();
                let ptr: *mut usize = (&mut len_value) as *mut usize;
                let buffer_ptr = LLVMBuildPointerCast(
                    codegen.builder,
                    value,
                    LLVMPointerType(LLVMInt8Type(), 0),
                    cstr_from_string(name).as_ptr(),
                );
                return Ok(Box::new(StringType {
                    name: name.to_string(),
                    length: ptr,
                    llvm_value: value,
                    llvm_value_pointer: Some(buffer_ptr),
                    str_value: val.to_string(), // fix
                }));
            }
        }
        Err(anyhow!("type is not a string"))
    }

    fn visit_bool(
        &mut self,
        left: &Expression,
        codegen: &LLVMCodegenBuilder,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::Bool(val) = left {
            let value = *val;
            let name = "bool_value";
            let bool_value = codegen.const_int(int1_type(), value.into(), 0);
            let alloca = codegen.build_alloca_store(bool_value, int1_type(), name);
            return Ok(Box::new(BoolType {
                name: name.parse()?,
                builder: codegen.builder,
                llvm_value: bool_value,
                llvm_value_pointer: alloca,
            }));
        }
        Err(anyhow!("type is not a bool"))
    }

    fn visit_variable(
        &mut self,
        left: &Expression,
        codegen: &LLVMCodegenBuilder,
        var_cache: &VariableCache,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::Variable(input) = left {
            return match codegen.current_function.symbol_table.get(input) {
                Some(val) => Ok(val.clone()),
                None => {
                    // check if variable is in function
                    // TODO: should this be reversed i.e check func var first then global
                    match var_cache.get(input) {
                        Some(val) => Ok(val),
                        None => Err(anyhow!(format!("Unknown variable {}", input))),
                    }
                }
            };
        }
        Err(anyhow!("type is not an i32"))
    }

    fn visit_binary(
        &mut self,
        _left: &Expression,
        _codegen: &LLVMCodegenBuilder,
    ) -> Result<Box<dyn TypeBase>> {
        unimplemented!()
    }

    fn visit_list(
        &mut self,
        left: &Expression,
        codegen: &mut LLVMCodegenBuilder,
        context: &mut ASTContext,
    ) -> Result<Box<dyn TypeBase>> {
        if let Expression::List(v) = left {
            let mut vec_expr = vec![];
            let mut visitor: Box<dyn Visitor<Box<dyn TypeBase>>> = Box::new(LLVMCodegenVisitor{});
            for x in v {
                let expr = context.match_ast(x.clone(), &mut visitor, codegen)?;
                vec_expr.push(expr)
            }
            let first_element = vec_expr.first().unwrap();
            let mut elements = vec![];
            for x in vec_expr.iter() {
                elements.push(x.get_value());
            }

            let array_type = first_element.get_llvm_type();
            let array_len = vec_expr.len() as u64;
            let llvm_array_value =
                codegen.const_array(array_type, elements.as_mut_ptr(), array_len);

            let llvm_array_type = codegen.array_type(array_type, array_len);
            let array_ptr = codegen.build_alloca_store(llvm_array_value, llvm_array_type, "array");
            return Ok(Box::new(ListType {
                llvm_value: llvm_array_value,
                llvm_value_ptr: array_ptr,
                llvm_type: llvm_array_type,
            }));
        }
        Err(anyhow!("unable to visit list"))
    }
}
