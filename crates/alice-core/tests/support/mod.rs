#![allow(dead_code)]
use alice_core::{
    BoxFuture,
    conversation::{ConversationId, ConversationLimits},
    message::{Message, MessageId, MessageLimits, Role},
    operation::{CancellationSignal, Correlation, OperationId},
    provider::{
        AiRequest, AiResponse, FinishReason, GenerationOptions, ModelRef, RequiredCapabilities,
        Usage,
    },
};
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    time::{Duration, SystemTime},
};

pub fn message(id: &str, role: Role, text: &str) -> Message {
    Message::new(
        MessageId::new(id).unwrap(),
        role,
        text,
        Some(SystemTime::UNIX_EPOCH),
        MessageLimits::new(2000, 8000).unwrap(),
    )
    .unwrap()
}
pub fn correlation() -> Correlation {
    Correlation::new(
        ConversationId::new("conversation-1").unwrap(),
        OperationId::new("operation-1").unwrap(),
        MessageId::new("input-1").unwrap(),
    )
}
pub fn model() -> ModelRef {
    ModelRef::new("instance-1", "namespace/model:1").unwrap()
}
pub fn request() -> AiRequest {
    AiRequest::new(
        correlation(),
        model(),
        vec![message("input-1", Role::User, "hola")],
        ConversationLimits::new(8, 16000).unwrap(),
        GenerationOptions {
            max_output_tokens: 64.try_into().unwrap(),
        },
        RequiredCapabilities {
            streaming: false,
            cancellation: false,
        },
    )
    .unwrap()
}
pub fn response() -> AiResponse {
    AiResponse::new(
        correlation(),
        model(),
        message("answer-1", Role::Assistant, "respuesta"),
        FinishReason::Stop,
        Usage {
            input_tokens: None,
            output_tokens: Some(0),
        },
    )
    .unwrap()
}
pub fn time(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

#[derive(Default)]
pub struct WakeCounter(pub AtomicUsize);
impl Wake for WakeCounter {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
pub fn poll<T>(
    future: Pin<&mut (dyn Future<Output = T> + Send + '_)>,
    counter: &Arc<WakeCounter>,
) -> Poll<T> {
    let waker = Waker::from(counter.clone());
    future.poll(&mut Context::from_waker(&waker))
}

/// Test-only one-shot double. No provider, executor, task, thread or I/O.
#[derive(Default)]
pub struct TestSignal(Mutex<SignalState>);
#[derive(Default)]
struct SignalState {
    requested: bool,
    next_id: u64,
    waiters: HashMap<u64, Waker>,
}
impl TestSignal {
    pub fn request(&self) -> bool {
        let wakeups = {
            let mut state = self.0.lock().unwrap();
            if state.requested {
                return false;
            }
            state.requested = true;
            std::mem::take(&mut state.waiters)
        };
        for waker in wakeups.into_values() {
            waker.wake();
        }
        true
    }
    pub fn waiters(&self) -> usize {
        self.0.lock().unwrap().waiters.len()
    }
}
struct Requested<'a> {
    signal: &'a TestSignal,
    registered: Option<u64>,
}
impl Future for Requested<'_> {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let signal = self.signal;
        let mut state = signal.0.lock().unwrap();
        if state.requested {
            return Poll::Ready(());
        }
        let id = match self.registered {
            Some(id) => id,
            None => {
                assert!(state.waiters.len() < 8, "bounded test fixture");
                let id = state.next_id;
                state.next_id = state.next_id.checked_add(1).unwrap();
                self.registered = Some(id);
                id
            }
        };
        // Observation and wakeup registration share the request's mutex: no gap.
        state.waiters.insert(id, cx.waker().clone());
        Poll::Pending
    }
}
impl Drop for Requested<'_> {
    fn drop(&mut self) {
        if let Some(id) = self.registered {
            self.signal.0.lock().unwrap().waiters.remove(&id);
        }
    }
}
impl CancellationSignal for TestSignal {
    fn is_requested(&self) -> bool {
        self.0.lock().unwrap().requested
    }
    fn requested(&self) -> BoxFuture<'_, ()> {
        Box::pin(Requested {
            signal: self,
            registered: None,
        })
    }
}
