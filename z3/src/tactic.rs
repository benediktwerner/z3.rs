use std::ffi::CString;
use z3_sys::*;
use ApplyResult;
use Context;
use Goal;
use Tactic;
use Z3_MUTEX;

impl<'ctx> Tactic<'ctx> {
    pub(crate) fn new(ctx: &'ctx Context, z3_tactic: Z3_tactic) -> Tactic<'ctx> {
        unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_tactic_inc_ref(ctx.z3_ctx, z3_tactic)
        };
        Tactic { ctx, z3_tactic }
    }

    pub fn from_name(ctx: &'ctx Context, name: &str) -> Option<Tactic<'ctx>> {
        let z3_tactic = unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            let name_cstring = CString::new(name).unwrap();
            let tactic = Z3_mk_tactic(ctx.z3_ctx, name_cstring.as_ptr());
            if tactic.is_null() {
                return None;
            }
            Z3_tactic_inc_ref(ctx.z3_ctx, tactic);
            tactic
        };
        Some(Tactic { ctx, z3_tactic })
    }

    pub fn apply(&self, goal: &Goal<'ctx>) -> ApplyResult<'ctx> {
        let result = unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_tactic_apply(self.ctx.z3_ctx, self.z3_tactic, goal.z3_goal)
        };
        ApplyResult::new(self.ctx, result)
    }
}

impl<'ctx> Drop for Tactic<'ctx> {
    fn drop(&mut self) {
        let guard = Z3_MUTEX.lock().unwrap();
        unsafe { Z3_tactic_dec_ref(self.ctx.z3_ctx, self.z3_tactic) };
    }
}
