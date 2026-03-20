//! 对话状态、打字机与内联等待。

use vn_runtime::command::{InlineEffect, InlineEffectKind};

use super::RenderState;

/// 对话状态
#[derive(Debug, Clone)]
pub struct DialogueState {
    /// 说话者名称（None 表示旁白）
    pub speaker: Option<String>,
    /// 对话内容（纯文本，标签已剥离）
    pub content: String,
    /// 当前可见字符数（用于打字机效果）
    pub visible_chars: usize,
    /// 是否显示完成
    pub is_complete: bool,
    /// 内联效果列表（位置索引到纯文本字符位置）
    pub inline_effects: Vec<InlineEffect>,
    /// 是否自动推进（行尾 `-->` 修饰符）
    pub no_wait: bool,
    /// 当前内联等待状态（打字机遇到 `{wait}` 后设置）
    pub inline_wait: Option<InlineWait>,
    /// 当前有效字速覆盖（由 `{speed}` 标签设置）
    pub effective_cps: Option<EffectiveCps>,
}

/// 打字机内联等待状态
#[derive(Debug, Clone)]
pub struct InlineWait {
    /// 剩余等待时间（None = 等待点击）
    pub remaining: Option<f64>,
}

/// 当前有效字速覆盖
#[derive(Debug, Clone)]
pub enum EffectiveCps {
    /// 绝对字速（字符/秒）
    Absolute(f64),
    /// 相对字速（基础速度的倍率）
    Relative(f64),
}

impl RenderState {
    /// 设置对话（立即显示全部，用于读档恢复等场景）
    pub fn set_dialogue(&mut self, speaker: Option<String>, content: String) {
        let visible_chars = content.chars().count();
        self.dialogue = Some(DialogueState {
            speaker,
            content,
            visible_chars,
            is_complete: true,
            inline_effects: vec![],
            no_wait: false,
            inline_wait: None,
            effective_cps: None,
        });
    }

    /// 开始打字机效果
    pub fn start_typewriter(
        &mut self,
        speaker: Option<String>,
        content: String,
        inline_effects: Vec<InlineEffect>,
        no_wait: bool,
    ) {
        self.dialogue = Some(DialogueState {
            speaker,
            content,
            visible_chars: 0,
            is_complete: false,
            inline_effects,
            no_wait,
            inline_wait: None,
            effective_cps: None,
        });
    }

    /// 推进打字机效果（返回是否完成）
    ///
    /// 推进一个字符后检查 inline_effects，触发等待或变速效果。
    pub fn advance_typewriter(&mut self) -> bool {
        if let Some(ref mut dialogue) = self.dialogue {
            let total_chars = dialogue.content.chars().count();
            if dialogue.visible_chars < total_chars {
                dialogue.visible_chars += 1;
                dialogue.is_complete = dialogue.visible_chars >= total_chars;
                // 在当前位置应用内联效果
                let pos = dialogue.visible_chars;
                for effect in &dialogue.inline_effects {
                    if effect.position == pos {
                        match &effect.kind {
                            InlineEffectKind::Wait(duration) => {
                                dialogue.inline_wait = Some(InlineWait {
                                    remaining: *duration,
                                });
                            }
                            InlineEffectKind::SetCpsAbsolute(n) => {
                                dialogue.effective_cps = Some(EffectiveCps::Absolute(*n));
                            }
                            InlineEffectKind::SetCpsRelative(m) => {
                                dialogue.effective_cps = Some(EffectiveCps::Relative(*m));
                            }
                            InlineEffectKind::ResetCps => {
                                dialogue.effective_cps = None;
                            }
                        }
                    }
                }
            }
            dialogue.is_complete
        } else {
            true
        }
    }

    /// 完成打字机效果（立即显示全部文本，跳过所有内联等待和变速）
    pub fn complete_typewriter(&mut self) {
        if let Some(ref mut dialogue) = self.dialogue {
            dialogue.visible_chars = dialogue.content.chars().count();
            dialogue.is_complete = true;
            dialogue.inline_wait = None;
            dialogue.effective_cps = None;
        }
    }

    /// 追加文本到当前对话（extend 命令）
    ///
    /// 打字机从当前位置继续，不重置 visible_chars。
    pub fn extend_dialogue(
        &mut self,
        content: &str,
        inline_effects: Vec<InlineEffect>,
        no_wait: bool,
    ) {
        if let Some(ref mut dialogue) = self.dialogue {
            let offset = dialogue.content.chars().count();
            dialogue.content.push_str(content);
            for mut effect in inline_effects {
                effect.position += offset;
                dialogue.inline_effects.push(effect);
            }
            dialogue.no_wait = no_wait;
            dialogue.is_complete = false;
        }
    }

    /// 检查当前位置是否有内联等待
    pub fn has_inline_wait(&self) -> bool {
        self.dialogue
            .as_ref()
            .is_some_and(|d| d.inline_wait.is_some())
    }

    /// 检查内联等待是否为点击等待（非定时）
    pub fn is_inline_click_wait(&self) -> bool {
        self.dialogue.as_ref().is_some_and(|d| {
            d.inline_wait
                .as_ref()
                .is_some_and(|w| w.remaining.is_none())
        })
    }

    /// 清除内联等待状态（点击跳过当前等待点）
    pub fn clear_inline_wait(&mut self) {
        if let Some(ref mut dialogue) = self.dialogue {
            dialogue.inline_wait = None;
        }
    }

    /// 更新内联定时等待（每帧调用），返回是否等待结束
    pub fn update_inline_wait(&mut self, dt: f32) -> bool {
        if let Some(ref mut dialogue) = self.dialogue
            && let Some(ref mut wait) = dialogue.inline_wait
            && let Some(ref mut remaining) = wait.remaining
        {
            *remaining -= dt as f64;
            if *remaining <= 0.0 {
                dialogue.inline_wait = None;
                return true;
            }
        }
        false
    }

    /// 获取当前有效字速（字符/秒），如果有覆盖则使用覆盖值
    pub fn effective_text_speed(&self, base_speed: f32) -> f32 {
        if let Some(ref dialogue) = self.dialogue {
            match &dialogue.effective_cps {
                Some(EffectiveCps::Absolute(n)) => *n as f32,
                Some(EffectiveCps::Relative(m)) => base_speed * *m as f32,
                None => base_speed,
            }
        } else {
            base_speed
        }
    }

    /// 清除对话
    pub fn clear_dialogue(&mut self) {
        self.dialogue = None;
    }

    /// 检查对话是否完成
    pub fn is_dialogue_complete(&self) -> bool {
        self.dialogue.as_ref().is_none_or(|d| d.is_complete)
    }
}
