use std::{
    ffi::c_void,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use whisper_rs::{FullParams, WhisperError, WhisperState};

struct Cancellation {
    expected: u64,
    authorization: Arc<AtomicU64>,
    deadline: std::time::Instant,
}

unsafe extern "C" fn aborted(data: *mut c_void) -> bool {
    // `run` retains this exact Cancellation allocation until the synchronous native call returns.
    let cancellation = &*(data as *const Cancellation);
    cancellation.authorization.load(Ordering::Acquire) != cancellation.expected
        || std::time::Instant::now() >= cancellation.deadline
}

pub fn run(
    state: &mut WhisperState,
    mut params: FullParams<'_, '_>,
    samples: &[f32],
    expected: u64,
    authorization: Arc<AtomicU64>,
) -> Result<(), WhisperError> {
    let mut cancellation = Cancellation {
        expected,
        authorization,
        deadline: std::time::Instant::now()
            + std::time::Duration::from_secs(dictation_core::defaults::INFERENCE_TIMEOUT_SECONDS),
    };
    // SAFETY: no native worker outlives state.full; the stack data is never moved during the call.
    unsafe {
        params.set_abort_callback(Some(aborted));
        params.set_abort_callback_user_data((&mut cancellation as *mut Cancellation).cast());
    }
    state.full(params, samples)
}
