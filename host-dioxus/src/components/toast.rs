use dioxus::prelude::*;

/// Toast 消息类型
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

/// 单条 Toast 消息
#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub text: String,
    pub toast_type: ToastType,
    pub id: u64,
}

/// Toast 队列状态
#[derive(Debug, Clone, Default)]
pub struct ToastQueue {
    pub messages: Vec<ToastMessage>,
    next_id: u64,
}

impl ToastQueue {
    pub fn push(&mut self, text: impl Into<String>, toast_type: ToastType) {
        self.messages.push(ToastMessage {
            text: text.into(),
            toast_type,
            id: self.next_id,
        });
        self.next_id += 1;
    }

    #[allow(dead_code)]
    pub fn success(&mut self, text: impl Into<String>) {
        self.push(text, ToastType::Success);
    }

    #[allow(dead_code)]
    pub fn error(&mut self, text: impl Into<String>) {
        self.push(text, ToastType::Error);
    }

    #[allow(dead_code)]
    pub fn info(&mut self, text: impl Into<String>) {
        self.push(text, ToastType::Info);
    }

    pub fn remove(&mut self, id: u64) {
        self.messages.retain(|m| m.id != id);
    }
}

/// Toast 渲染层（右上角固定）
#[component]
pub fn ToastLayer() -> Element {
    let mut toast_queue = use_context::<Signal<ToastQueue>>();
    let messages = toast_queue.read().messages.clone();

    if messages.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "toast-layer",
            for msg in &messages {
                {
                    let id = msg.id;
                    let text = msg.text.clone();
                    let modifier = match msg.toast_type {
                        ToastType::Info => "toast--info",
                        ToastType::Success => "toast--success",
                        ToastType::Warning => "toast--warning",
                        ToastType::Error => "toast--error",
                    };

                    rsx! {
                        div {
                            key: "{id}",
                            class: "toast {modifier}",
                            onanimationend: move |_| {
                                toast_queue.write().remove(id);
                            },
                            "{text}"
                        }
                    }
                }
            }
        }
    }
}
