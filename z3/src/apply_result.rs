use ast;
use std::ffi::CStr;
use std::fmt;
use z3_sys::*;
use ApplyResult;
use Context;
use Goal;
use Z3_MUTEX;

impl<'ctx> ApplyResult<'ctx> {
    pub(crate) fn new(ctx: &'ctx Context, result: Z3_apply_result) -> ApplyResult<'ctx> {
        unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_apply_result_inc_ref(ctx.z3_ctx, result);
        };
        ApplyResult {
            ctx,
            z3_result: result,
        }
    }

    pub fn len(&self) -> usize {
        let len = unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_apply_result_get_num_subgoals(self.ctx.z3_ctx, self.z3_result)
        };
        len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, index: usize) -> Option<Goal<'ctx>> {
        if index >= self.len() {
            return None;
        }
        let goal = unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_apply_result_get_subgoal(
                self.ctx.z3_ctx,
                self.z3_result,
                index as ::std::os::raw::c_uint,
            )
        };
        Some(Goal::new(self.ctx, goal))
    }

    pub fn as_expr(&self) -> ast::Bool<'ctx> {
        if let Some(goal) = self.get(0) {
            let mut expr = goal.as_expr();
            for i in 1..self.len() {
                expr = expr.or(&[&self.get(i).unwrap().as_expr()]);
            }
            expr
        } else {
            ast::Bool::from_bool(self.ctx, false)
        }
    }
}

impl<'ctx> fmt::Display for ApplyResult<'ctx> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let p = unsafe { Z3_apply_result_to_string(self.ctx.z3_ctx, self.z3_result) };
        if p.is_null() {
            return Result::Err(fmt::Error);
        }
        match unsafe { CStr::from_ptr(p) }.to_str() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => Result::Err(fmt::Error),
        }
    }
}

impl<'ctx> Drop for ApplyResult<'ctx> {
    fn drop(&mut self) {
        let guard = Z3_MUTEX.lock().unwrap();
        unsafe { Z3_apply_result_dec_ref(self.ctx.z3_ctx, self.z3_result) };
    }
}
