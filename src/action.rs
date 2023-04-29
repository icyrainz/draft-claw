use std::{pin::Pin, future::Future};

struct HelpAnError {}

pub fn wrap_action<
    C: Fn() -> F + 'static,
    F: Future<Output = ()> + 'static,
>(
    do_work_hard: C,
) -> impl Fn() -> Pin<Box<dyn Future<Output = ()>>> {
    move || {
        let fut = do_work_hard();
        Box::pin(async move {
            println!("I'm doin my chores Ma, I promise!");
            // **help** - if I uncomment this it fails!
            drop(fut.await);
            println!("Yay! The chores are now complit.");
        })
    }
}

pub struct Action {
    pub cmd: &'static str,
    pub desc: &'static str,
    pub action:Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>>>, 
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

