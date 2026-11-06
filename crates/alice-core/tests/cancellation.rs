mod support;
use alice_core::{
    BoxFuture,
    operation::{CancellationSignal, OperationContext},
};
use std::{
    sync::{Arc, atomic::Ordering},
    task::Poll,
    time::{Duration, Instant},
};
use support::{TestSignal, WakeCounter, poll};

#[test]
fn local_box_future_uses_only_standard_future_polling() {
    let mut future: BoxFuture<'static, u32> = Box::pin(std::future::ready(7));
    assert_eq!(
        poll(future.as_mut(), &Arc::new(WakeCounter::default())),
        Poll::Ready(7)
    );
}
#[test]
fn cancellation_before_waiting_resolves_immediately_and_is_one_shot() {
    let signal = TestSignal::default();
    assert!(!signal.is_requested());
    assert!(signal.request());
    assert!(!signal.request());
    assert!(signal.is_requested());
    let mut wait = signal.requested();
    assert_eq!(
        poll(wait.as_mut(), &Arc::new(WakeCounter::default())),
        Poll::Ready(())
    );
    assert_eq!(signal.waiters(), 0);
}
#[test]
fn request_between_future_creation_and_first_poll_is_not_lost() {
    let signal = TestSignal::default();
    let mut wait = signal.requested();
    signal.request();
    assert_eq!(
        poll(wait.as_mut(), &Arc::new(WakeCounter::default())),
        Poll::Ready(())
    );
}
#[test]
fn pending_waiter_is_woken_and_then_resolves() {
    let signal = TestSignal::default();
    let counter = Arc::new(WakeCounter::default());
    let mut wait = signal.requested();
    assert_eq!(poll(wait.as_mut(), &counter), Poll::Pending);
    assert_eq!(signal.waiters(), 1);
    signal.request();
    assert_eq!(counter.0.load(Ordering::SeqCst), 1);
    assert_eq!(poll(wait.as_mut(), &counter), Poll::Ready(()));
}
#[test]
fn re_poll_updates_waker_without_duplicate_registration() {
    let signal = TestSignal::default();
    let old = Arc::new(WakeCounter::default());
    let latest = Arc::new(WakeCounter::default());
    let mut wait = signal.requested();
    assert_eq!(poll(wait.as_mut(), &old), Poll::Pending);
    assert_eq!(poll(wait.as_mut(), &latest), Poll::Pending);
    assert_eq!(signal.waiters(), 1);
    signal.request();
    assert_eq!(old.0.load(Ordering::SeqCst), 0);
    assert_eq!(latest.0.load(Ordering::SeqCst), 1);
}
#[test]
fn multiple_waiters_and_dropped_futures_do_not_lose_or_revoke_request() {
    let signal = TestSignal::default();
    let a = Arc::new(WakeCounter::default());
    let b = Arc::new(WakeCounter::default());
    let mut first = signal.requested();
    let mut second = signal.requested();
    assert_eq!(poll(first.as_mut(), &a), Poll::Pending);
    assert_eq!(poll(second.as_mut(), &b), Poll::Pending);
    drop(first);
    assert_eq!(signal.waiters(), 1);
    signal.request();
    assert_eq!(a.0.load(Ordering::SeqCst), 0);
    assert_eq!(b.0.load(Ordering::SeqCst), 1);
    drop(second);
    let mut third = signal.requested();
    assert_eq!(poll(third.as_mut(), &b), Poll::Ready(()));
    assert!(signal.is_requested());
}
#[test]
fn request_wakes_all_registered_waiters() {
    let signal = TestSignal::default();
    let counter = Arc::new(WakeCounter::default());
    let mut first = signal.requested();
    let mut second = signal.requested();
    assert_eq!(poll(first.as_mut(), &counter), Poll::Pending);
    assert_eq!(poll(second.as_mut(), &counter), Poll::Pending);
    signal.request();
    assert_eq!(counter.0.load(Ordering::SeqCst), 2);
    assert_eq!(poll(first.as_mut(), &counter), Poll::Ready(()));
    assert_eq!(poll(second.as_mut(), &counter), Poll::Ready(()));
}
#[test]
fn context_observes_request_and_explicit_clock_without_runtime() {
    let signal = Arc::new(TestSignal::default());
    let start = Instant::now();
    let context = OperationContext::new(
        support::correlation().operation_id().clone(),
        start + Duration::from_secs(1),
        signal.clone(),
    );
    assert!(!context.is_expired_at(start));
    assert!(context.is_expired_at(context.deadline()));
    assert!(support::request().validate_context(&context).is_ok());
    let wrong = OperationContext::new("other".parse().unwrap(), context.deadline(), signal.clone());
    assert!(support::request().validate_context(&wrong).is_err());
    signal.request();
    assert!(context.cancellation().is_requested());
}
