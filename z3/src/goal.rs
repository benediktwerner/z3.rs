use ast;
use ast::Ast;
use std::ffi::CStr;
use std::fmt;
use z3_sys::*;
use Context;
use Goal;
use Z3_MUTEX;

impl<'ctx> Goal<'ctx> {
    pub(crate) fn new(ctx: &'ctx Context, z3_goal: Z3_goal) -> Goal<'ctx> {
        unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_goal_inc_ref(ctx.z3_ctx, z3_goal)
        };
        Goal { ctx, z3_goal }
    }

    pub fn empty(ctx: &'ctx Context) -> Goal<'ctx> {
        Goal::emtpy_ex(ctx, true, false, false)
    }

    pub fn emtpy_ex(
        ctx: &'ctx Context,
        models: bool,
        unsat_cores: bool,
        proofs: bool,
    ) -> Goal<'ctx> {
        Goal {
            ctx,
            z3_goal: unsafe {
                let guard = Z3_MUTEX.lock().unwrap();
                let goal = Z3_mk_goal(ctx.z3_ctx, models, unsat_cores, proofs);
                Z3_goal_inc_ref(ctx.z3_ctx, goal);
                goal
            },
        }
    }

    pub fn assert(&self, ast: &ast::Bool<'ctx>) {
        let guard = Z3_MUTEX.lock().unwrap();
        unsafe { Z3_goal_assert(self.ctx.z3_ctx, self.z3_goal, ast.z3_ast) }
    }

    pub fn len(&self) -> usize {
        let len = unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_goal_size(self.ctx.z3_ctx, self.z3_goal)
        };
        len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<ast::Bool<'ctx>> {
        if index >= self.len() {
            return None;
        }
        let formula = unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_goal_formula(
                self.ctx.z3_ctx,
                self.z3_goal,
                index as ::std::os::raw::c_uint,
            )
        };
        Some(ast::Bool::new(self.ctx, formula))
    }

    pub fn as_expr(&self) -> ast::Bool<'ctx> {
        if let Some(mut expr) = self.get(0) {
            for i in 1..self.len() {
                expr = expr.and(&[&self.get(i).unwrap()]);
            }
            expr
        } else {
            ast::Bool::from_bool(self.ctx, true)
        }
    }
}

impl<'ctx> From<&ast::Bool<'ctx>> for Goal<'ctx> {
    fn from(b: &ast::Bool<'ctx>) -> Goal<'ctx> {
        let goal = Goal::empty(b.ctx);
        goal.assert(b);
        goal
    }
}

impl<'ctx> fmt::Display for Goal<'ctx> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let p = unsafe { Z3_goal_to_string(self.ctx.z3_ctx, self.z3_goal) };
        if p.is_null() {
            return Result::Err(fmt::Error);
        }
        match unsafe { CStr::from_ptr(p) }.to_str() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => Result::Err(fmt::Error),
        }
    }
}

impl<'ctx> Drop for Goal<'ctx> {
    fn drop(&mut self) {
        let guard = Z3_MUTEX.lock().unwrap();
        unsafe { Z3_goal_dec_ref(self.ctx.z3_ctx, self.z3_goal) };
    }
}
