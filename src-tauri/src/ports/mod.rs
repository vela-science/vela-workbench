pub(crate) mod entire;
pub(crate) mod evidence;
pub(crate) mod git;
pub(crate) mod launch;
pub(crate) mod native_exec;
pub(crate) mod opengauss;
pub(crate) mod problem_handoff;
mod process;
pub(crate) mod tranche_three;
pub(crate) mod vela;

pub(crate) use process::{
    CancellableProcessOutput, PortError, ProcessOutput, ProcessSpec, ensure_not_truncated,
    environment_summary, run_bounded, run_cancellable,
};

pub(crate) fn valid_recovery_operation_id(value: &str) -> bool {
    value.strip_prefix("vop_").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}
