use std::rc::Rc;
use widen::{Widen, Widening};

#[test]
fn preserves_success_and_owns_the_original_error() {
    let success: Result<_, Widening<String, ()>> = Result::<_, String>::Ok(42).widen();
    assert_eq!(success.unwrap(), 42);

    let error = String::from("original error");
    let address = error.as_ptr();
    let result: Result<(), Widening<_, ()>> = Err(error).widen();
    let error = result.unwrap_err().into_inner();
    assert_eq!(error, "original error");
    assert_eq!(error.as_ptr(), address);
}

#[test]
fn marker_adds_no_storage_or_send_sync_requirements() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Widening<u64, Rc<()>>>();
    assert_eq!(size_of::<Widening<u64, Rc<()>>>(), size_of::<u64>());
}

#[test]
fn debug_does_not_require_target_to_implement_debug() {
    struct Target;
    let error = Widening::<_, Target>::new(42);
    assert_eq!(format!("{error:?}"), "Widening(42)");
}
