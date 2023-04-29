use std::{future::Future, pin::Pin};

pub fn wrap_action<'a, C: Fn() -> F + 'static, F: Future<Output = ()> + 'static>(
    action: C,
) -> impl Fn() -> Pin<Box<dyn Future<Output = ()>>> + 'a {
    move || {
        let fut = action();
        Box::pin(async move {
            drop(fut.await);
        })
    }
}

pub struct Action {
    pub cmd: &'static str,
    pub desc: &'static str,
    pub action: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>>>,
}

impl Action {
    pub fn new(
        cmd: &'static str,
        desc: &'static str,
        action: impl Fn() -> Pin<Box<dyn Future<Output = ()>>> + 'static,
    ) -> Self {
        Self {
            cmd,
            desc,
            action: Box::new(action),
        }
    }

    pub async fn invoke(&self) {
        (self.action)().await;
    }
}
