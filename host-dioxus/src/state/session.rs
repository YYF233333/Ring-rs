use crate::error::{HostError, HostResult};
use crate::render_state::HostScreen;

use super::*;

impl AppStateInner {
    pub fn frontend_connected(&mut self, label: Option<String>) -> FrontendSession {
        self.session.next_client_id += 1;
        let token = format!("client-{}", self.session.next_client_id);
        let label = label.unwrap_or_else(|| "frontend".to_string());
        self.session.client_owner = Some(SessionOwner {
            token: token.clone(),
            label: label.clone(),
        });

        if self.runtime.is_none() {
            self.host_screen = HostScreen::Title;
        }
        self.project_render_state();

        FrontendSession {
            client_token: token,
            render_state: self.render_state.clone(),
        }
    }

    pub fn assert_owner(&self, client_token: &str) -> HostResult<()> {
        let owner = self.session.client_owner.as_ref().ok_or_else(|| {
            HostError::Session(
                "当前没有已连接的前端 owner，请先调用 frontend_connected".to_string(),
            )
        })?;
        if owner.token == client_token {
            Ok(())
        } else {
            Err(HostError::Session(format!(
                "当前会话已被其他客户端接管（owner={}），拒绝推进请求",
                owner.label
            )))
        }
    }
}
