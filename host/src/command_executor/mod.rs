//! # Command Executor 模块
//!
//! Command 执行器，负责将 Runtime 发出的 Command 转换为实际操作。
//!
//! ## 设计说明
//!
//! - `CommandExecutor` 接收 `Command`，更新 `RenderState` 和控制音频
//! - 执行器不直接渲染，只更新状态，渲染由 `Renderer` 负责
//! - 支持过渡效果的执行（通过 `TransitionState` 管理）

use vn_runtime::command::{Command, Choice, Position, Transition, TransitionArg};
use crate::renderer::{RenderState, ChoiceItem, SceneMaskState, SceneMaskType};
use crate::resources::ResourceManager;

/// Command 执行结果
#[derive(Debug, Clone, PartialEq)]
pub enum ExecuteResult {
    /// 执行成功，继续
    Ok,
    /// 执行成功，需要等待用户输入（对话显示完成后）
    WaitForClick,
    /// 执行成功，需要等待用户选择
    WaitForChoice { choice_count: usize },
    /// 执行成功，需要等待指定时长（毫秒）
    WaitForTime(u64),
    /// 资源加载中
    Loading,
    /// 执行失败
    Error(String),
}

/// 音频命令
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// 播放 BGM
    PlayBgm {
        path: String,
        looping: bool,
        fade_in: Option<f32>,
    },
    /// 停止 BGM
    StopBgm {
        fade_out: Option<f32>,
    },
    /// 播放 SFX
    PlaySfx {
        path: String,
    },
}

/// 过渡效果信息
#[derive(Debug, Clone, Default)]
pub struct TransitionInfo {
    /// 是否有背景过渡
    pub has_background_transition: bool,
    /// 旧背景路径
    pub old_background: Option<String>,
    /// 过渡效果
    pub transition: Option<vn_runtime::command::Transition>,
}

/// 命令执行输出
#[derive(Debug, Clone, Default)]
pub struct CommandOutput {
    /// 执行结果
    pub result: ExecuteResult,
    /// 过渡信息
    pub transition_info: TransitionInfo,
    /// 音频命令（如果有）
    pub audio_command: Option<AudioCommand>,
}

impl Default for ExecuteResult {
    fn default() -> Self {
        Self::Ok
    }
}

/// Command 执行器
///
/// 负责将 Runtime 发出的 Command 转换为实际的渲染状态更新。
#[derive(Debug)]
pub struct CommandExecutor {
    /// 当前是否有活跃的过渡效果
    transition_active: bool,
    /// 过渡效果计时器
    transition_timer: f32,
    /// 过渡效果总时长
    transition_duration: f32,
    /// 最近一次执行的输出
    pub last_output: CommandOutput,
}

impl CommandExecutor {
    /// 创建新的 Command 执行器
    pub fn new() -> Self {
        Self {
            transition_active: false,
            transition_timer: 0.0,
            transition_duration: 0.0,
            last_output: CommandOutput::default(),
        }
    }

    /// 执行单个 Command
    ///
    /// 根据 Command 类型更新 RenderState。
    /// 返回执行结果，同时更新 `last_output` 以获取过渡和音频信息。
    pub fn execute(
        &mut self,
        command: &Command,
        render_state: &mut RenderState,
        _resource_manager: &ResourceManager,
    ) -> ExecuteResult {
        // 重置输出
        self.last_output = CommandOutput::default();

        let result = match command {
            Command::ShowBackground { path, transition } => {
                self.execute_show_background(path, transition.clone(), render_state)
            }
            Command::ChangeScene { path, transition } => {
                // ChangeScene 是复合场景切换，包含：清立绘、换背景、遮罩过渡
                self.execute_change_scene(path, transition.clone(), render_state, _resource_manager)
            }
            Command::ShowCharacter { path, alias, position, transition } => {
                self.execute_show_character(path, alias, *position, transition, render_state)
            }
            Command::HideCharacter { alias, transition } => {
                self.execute_hide_character(alias, transition, render_state)
            }
            Command::ShowText { speaker, content } => {
                self.execute_show_text(speaker.clone(), content, render_state)
            }
            Command::PresentChoices { style, choices } => {
                self.execute_present_choices(style.clone(), choices, render_state)
            }
            Command::ChapterMark { title, level } => {
                self.execute_chapter_mark(title, *level, render_state)
            }
            Command::PlayBgm { path, looping } => {
                self.execute_play_bgm(path, *looping)
            }
            Command::StopBgm { fade_out } => {
                self.execute_stop_bgm(*fade_out)
            }
            Command::PlaySfx { path } => {
                self.execute_play_sfx(path)
            }
            Command::UIAnimation { effect } => {
                self.execute_ui_animation(effect)
            }
        };

        self.last_output.result = result.clone();
        result
    }

    /// 批量执行 Commands
    ///
    /// 执行一组 Commands，返回最后一个需要等待的结果。
    pub fn execute_batch(
        &mut self,
        commands: &[Command],
        render_state: &mut RenderState,
        resource_manager: &ResourceManager,
    ) -> ExecuteResult {
        let mut last_result = ExecuteResult::Ok;

        for command in commands {
            let result = self.execute(command, render_state, resource_manager);
            
            // 记录需要等待的结果
            match &result {
                ExecuteResult::WaitForClick
                | ExecuteResult::WaitForChoice { .. }
                | ExecuteResult::WaitForTime(_) => {
                    last_result = result;
                }
                ExecuteResult::Error(_) => {
                    return result; // 遇到错误立即返回
                }
                _ => {}
            }
        }

        last_result
    }

    /// 执行 ShowBackground
    fn execute_show_background(
        &mut self,
        path: &str,
        transition: Option<Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 保存旧背景用于过渡效果
        let old_background = render_state.current_background.clone();

        // 设置新背景路径
        render_state.set_background(path.to_string());

        // 记录过渡信息
        self.last_output.transition_info = TransitionInfo {
            has_background_transition: true,
            old_background,
            transition: transition.clone(),
        };

        // 处理过渡效果
        if let Some(ref trans) = transition {
            self.start_transition(trans);
        }

        ExecuteResult::Ok
    }

    /// 执行 ChangeScene（复合场景切换）
    ///
    /// 与 ShowBackground 不同，ChangeScene 会：
    /// 1. 隐藏 UI
    /// 2. 清除所有立绘
    /// 3. 使用遮罩过渡效果切换背景
    /// 4. 恢复 UI
    fn execute_change_scene(
        &mut self,
        path: &str,
        transition: Option<Transition>,
        render_state: &mut RenderState,
        resource_manager: &ResourceManager,
    ) -> ExecuteResult {
        // 保存旧背景用于过渡效果
        let old_background = render_state.current_background.clone();

        // 1. 隐藏 UI（对话框、选择分支等）
        render_state.ui_visible = false;

        // 2. 清除所有立绘
        render_state.hide_all_characters();

        // 3. 根据 transition 类型设置遮罩/过渡
        if let Some(ref trans) = transition {
            let name_lower = trans.name.to_lowercase();
            let duration = trans.get_duration().unwrap_or(0.5) as f32;

            match name_lower.as_str() {
                "fade" => {
                    // 黑屏遮罩
                    let mut mask = SceneMaskState::new(
                        SceneMaskType::SolidBlack,
                        duration,
                    );
                    mask.set_pending_background(path.to_string());
                    render_state.scene_mask = Some(mask);
                    println!("🎬 changeScene: Fade 黑屏过渡 ({}s)", duration);
                }
                "fadewhite" => {
                    // 白屏遮罩
                    let mut mask = SceneMaskState::new(
                        SceneMaskType::SolidWhite,
                        duration,
                    );
                    mask.set_pending_background(path.to_string());
                    render_state.scene_mask = Some(mask);
                    println!("🎬 changeScene: FadeWhite 白屏过渡 ({}s)", duration);
                }
                "rule" => {
                    // 图片遮罩 - 使用 resource_manager 规范化路径
                    let raw_mask_path = trans.get_named("mask")
                        .and_then(|arg| {
                            if let TransitionArg::String(s) = arg {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    
                    // 规范化路径：相对路径需要基于脚本目录解析
                    // 注意：这里的 raw_mask_path 是相对于脚本文件的路径
                    // 需要与背景路径 path 使用相同的基准目录
                    let normalized_mask_path = resource_manager.resolve_path(&raw_mask_path);
                    let reversed = trans.get_reversed().unwrap_or(false);
                    
                    let mut mask = SceneMaskState::new(
                        SceneMaskType::Rule { mask_path: normalized_mask_path.clone(), reversed },
                        duration,
                    );
                    mask.set_pending_background(path.to_string());
                    render_state.scene_mask = Some(mask);
                    println!("🎬 changeScene: Rule 遮罩过渡 ({}, {}s, reversed={})", normalized_mask_path, duration, reversed);
                }
                "dissolve" => {
                    // Dissolve 使用 TransitionManager 处理背景过渡
                    // 记录过渡信息，让 main.rs 启动背景过渡
                    self.last_output.transition_info = TransitionInfo {
                        has_background_transition: true,
                        old_background: old_background.clone(),
                        transition: transition.clone(),
                    };
                    // 立即切换背景（交叉溶解依赖 old_background）
                    render_state.set_background(path.to_string());
                    // 立即恢复 UI
                    render_state.ui_visible = true;
                    println!("🎬 changeScene: Dissolve 过渡 ({}s)", duration);
                }
                _ => {
                    // 未知效果，使用默认 dissolve
                    self.last_output.transition_info = TransitionInfo {
                        has_background_transition: true,
                        old_background: old_background.clone(),
                        transition: transition.clone(),
                    };
                    render_state.set_background(path.to_string());
                    render_state.ui_visible = true;
                    println!("🎬 changeScene: 未知效果 '{}', 使用 dissolve", trans.name);
                }
            }
        } else {
            // 无过渡效果，立即恢复 UI
            render_state.set_background(path.to_string());
            render_state.ui_visible = true;
        }

        // 注意：对于 Fade/FadeWhite/Rule 效果，不设置 has_background_transition
        // 因为这些效果使用 SceneMaskState 处理，而不是 TransitionManager

        ExecuteResult::Ok
    }

    /// 执行 ShowCharacter
    fn execute_show_character(
        &mut self,
        path: &str,
        alias: &str,
        position: Position,
        transition: &Option<Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 解析过渡效果持续时间
        // 如果 transition 存在且是 dissolve/fade，使用指定的 duration 或默认 0.3 秒
        let transition_duration = transition.as_ref().and_then(|t| {
            let name_lower = t.name.to_lowercase();
            if name_lower == "dissolve" || name_lower == "fade" {
                // 如果有指定 duration 则使用，否则使用默认值 0.3 秒
                Some(t.get_duration().map(|d| d as f32).unwrap_or(0.3))
            } else {
                None
            }
        });

        // 显示角色（带过渡效果）
        render_state.show_character_with_transition(
            alias.to_string(),
            path.to_string(),
            position,
            transition_duration,
        );

        ExecuteResult::Ok
    }

    /// 执行 HideCharacter
    fn execute_hide_character(
        &mut self,
        alias: &str,
        transition: &Option<Transition>,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 解析过渡效果持续时间
        // 如果 transition 存在且是 dissolve/fade，使用指定的 duration 或默认 0.3 秒
        let transition_duration = transition.as_ref().and_then(|t| {
            let name_lower = t.name.to_lowercase();
            if name_lower == "dissolve" || name_lower == "fade" {
                // 如果有指定 duration 则使用，否则使用默认值 0.3 秒
                Some(t.get_duration().map(|d| d as f32).unwrap_or(0.3))
            } else {
                None
            }
        });

        // 隐藏角色（带过渡效果）
        render_state.hide_character_with_transition(alias, transition_duration);

        ExecuteResult::Ok
    }

    /// 执行 ShowText
    fn execute_show_text(
        &mut self,
        speaker: Option<String>,
        content: &str,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 清除章节标记（避免遮挡对话）
        render_state.clear_chapter_mark();

        // 开始打字机效果
        render_state.start_typewriter(speaker, content.to_string());

        // ShowText 通常需要等待用户点击
        ExecuteResult::WaitForClick
    }

    /// 执行 PresentChoices
    fn execute_present_choices(
        &mut self,
        style: Option<String>,
        choices: &[Choice],
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 清除对话框和章节标记
        render_state.clear_dialogue();
        render_state.clear_chapter_mark();

        // 转换选项格式
        let choice_items: Vec<ChoiceItem> = choices
            .iter()
            .map(|c| ChoiceItem {
                text: c.text.clone(),
                target_label: c.target_label.clone(),
            })
            .collect();

        let choice_count = choice_items.len();

        // 设置选择界面
        render_state.set_choices(choice_items, style);

        ExecuteResult::WaitForChoice { choice_count }
    }

    /// 执行 ChapterMark
    fn execute_chapter_mark(
        &mut self,
        title: &str,
        level: u8,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        // 清除其他 UI 元素
        render_state.clear_dialogue();
        render_state.clear_choices();

        // 显示章节标记
        render_state.set_chapter_mark(title.to_string(), level);

        // 章节标记通常需要等待用户点击
        ExecuteResult::WaitForClick
    }

    /// 执行 PlayBgm
    fn execute_play_bgm(&mut self, path: &str, looping: bool) -> ExecuteResult {
        // 记录音频命令，由 main.rs 处理实际播放
        self.last_output.audio_command = Some(AudioCommand::PlayBgm {
            path: path.to_string(),
            looping,
            fade_in: Some(0.5), // 默认 0.5 秒淡入
        });
        println!("🎵 命令：播放 BGM: {} (循环: {})", path, looping);
        ExecuteResult::Ok
    }

    /// 执行 StopBgm
    fn execute_stop_bgm(&mut self, fade_out: Option<f64>) -> ExecuteResult {
        // 记录音频命令
        self.last_output.audio_command = Some(AudioCommand::StopBgm {
            fade_out: fade_out.map(|d| d as f32),
        });
        if let Some(duration) = fade_out {
            println!("🎵 命令：停止 BGM (淡出: {}s)", duration);
        } else {
            println!("🎵 命令：停止 BGM (立即)");
        }
        ExecuteResult::Ok
    }

    /// 执行 PlaySfx
    fn execute_play_sfx(&mut self, path: &str) -> ExecuteResult {
        // 记录音频命令
        self.last_output.audio_command = Some(AudioCommand::PlaySfx {
            path: path.to_string(),
        });
        println!("🔊 命令：播放音效: {}", path);
        ExecuteResult::Ok
    }

    /// 执行 UIAnimation
    fn execute_ui_animation(&mut self, effect: &Transition) -> ExecuteResult {
        // TODO: 实现 UI 动画
        println!("✨ UI 动画: {} {:?}", effect.name, effect.args);
        self.start_transition(effect);
        ExecuteResult::Ok
    }

    /// 开始过渡效果
    fn start_transition(&mut self, transition: &Transition) {
        self.transition_active = true;
        self.transition_timer = 0.0;

        // 从参数中提取时长，默认 0.3 秒（优先命名参数，回退位置参数）
        self.transition_duration = transition.get_duration().map(|d| d as f32).unwrap_or(0.3);

        println!("🎬 开始过渡效果: {} ({}s)", transition.name, self.transition_duration);
    }

    /// 更新过渡效果
    ///
    /// 返回 true 表示过渡效果仍在进行中。
    pub fn update_transition(&mut self, dt: f32) -> bool {
        if !self.transition_active {
            return false;
        }

        self.transition_timer += dt;
        if self.transition_timer >= self.transition_duration {
            self.transition_active = false;
            self.transition_timer = 0.0;
            println!("🎬 过渡效果完成");
            return false;
        }

        true
    }

    /// 获取过渡效果进度 (0.0 - 1.0)
    pub fn get_transition_progress(&self) -> f32 {
        if !self.transition_active || self.transition_duration <= 0.0 {
            return 1.0;
        }
        (self.transition_timer / self.transition_duration).min(1.0)
    }

    /// 检查是否有活跃的过渡效果
    pub fn is_transition_active(&self) -> bool {
        self.transition_active
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = CommandExecutor::new();
        assert!(!executor.is_transition_active());
    }

    #[test]
    fn test_execute_show_text() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::ShowText {
            speaker: Some("北风".to_string()),
            content: "你好".to_string(),
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::WaitForClick);
        assert!(render_state.dialogue.is_some());
    }

    #[test]
    fn test_execute_present_choices() {
        let mut executor = CommandExecutor::new();
        let mut render_state = RenderState::new();
        let resource_manager = ResourceManager::new("assets", 256);

        let cmd = Command::PresentChoices {
            style: None,
            choices: vec![
                Choice { text: "选项1".to_string(), target_label: "label1".to_string() },
                Choice { text: "选项2".to_string(), target_label: "label2".to_string() },
            ],
        };

        let result = executor.execute(&cmd, &mut render_state, &resource_manager);
        assert_eq!(result, ExecuteResult::WaitForChoice { choice_count: 2 });
        assert!(render_state.choices.is_some());
    }
}
