//! 背景、立绘、章节标记与选项等场景显示状态。

use vn_runtime::command::Position;

use super::super::character_animation::AnimatableCharacter;
use super::{
    ChapterMarkPhase, ChapterMarkState, CharacterSprite, ChoiceItem, ChoicesState, RenderState,
};

impl RenderState {
    /// 设置背景
    pub fn set_background(&mut self, path: String) {
        self.current_background = Some(path);
    }

    /// 清除背景
    pub fn clear_background(&mut self) {
        self.current_background = None;
    }

    /// 显示角色
    ///
    /// 创建角色数据和动画状态。初始透明度为 0，需要通过动画系统淡入。
    ///
    /// # 返回
    /// 返回角色的动画状态引用，可用于注册到动画系统
    pub fn show_character(
        &mut self,
        alias: String,
        texture_path: String,
        position: Position,
    ) -> &AnimatableCharacter {
        let z_order = self.visible_characters.len() as i32;

        self.visible_characters.insert(
            alias.clone(),
            CharacterSprite {
                texture_path,
                position,
                z_order,
                fading_out: false,
                anim: AnimatableCharacter::transparent(&alias), // 初始透明，等待淡入
            },
        );

        &self.visible_characters.get(&alias).unwrap().anim
    }

    /// 获取角色的动画状态
    pub fn get_character_anim(&self, alias: &str) -> Option<&AnimatableCharacter> {
        self.visible_characters.get(alias).map(|c| &c.anim)
    }

    /// 获取角色的动画状态（可变）
    pub fn get_character_anim_mut(&mut self, alias: &str) -> Option<&mut AnimatableCharacter> {
        self.visible_characters.get_mut(alias).map(|c| &mut c.anim)
    }

    /// 隐藏角色（立即移除）
    pub fn hide_character(&mut self, alias: &str) {
        self.visible_characters.remove(alias);
    }

    /// 标记角色为淡出状态
    ///
    /// 角色会在动画完成后被 `remove_fading_out_characters` 移除。
    pub fn mark_character_fading_out(&mut self, alias: &str) {
        if let Some(character) = self.visible_characters.get_mut(alias) {
            character.fading_out = true;
        }
    }

    /// 移除所有标记为淡出且动画已完成的角色
    ///
    /// 应在动画系统更新后调用，传入已完成淡出的角色列表。
    pub fn remove_fading_out_characters(&mut self, completed_aliases: &[String]) {
        for alias in completed_aliases {
            if let Some(character) = self.visible_characters.get(alias)
                && character.fading_out
            {
                self.visible_characters.remove(alias);
            }
        }
    }

    /// 隐藏所有角色
    pub fn hide_all_characters(&mut self) {
        self.visible_characters.clear();
    }

    /// 设置章节标记（覆盖策略：新的直接覆盖旧的）
    ///
    /// 从 FadeIn 阶段开始，alpha = 0，由 update_chapter_mark 驱动动画。
    pub fn set_chapter_mark(&mut self, title: String, level: u8) {
        self.chapter_mark = Some(ChapterMarkState {
            title,
            level,
            alpha: 0.0,
            timer: 0.0,
            phase: ChapterMarkPhase::FadeIn,
        });
    }

    /// 清除章节标记
    pub fn clear_chapter_mark(&mut self) {
        self.chapter_mark = None;
    }

    /// 更新章节标记动画（由每帧 update 调用）
    ///
    /// 返回 true 表示章节标记仍在显示。
    /// 此更新**不受用户快进/点击影响**。
    pub fn update_chapter_mark(&mut self, dt: f32) -> bool {
        let should_clear = if let Some(ref mut mark) = self.chapter_mark {
            mark.timer += dt;
            match mark.phase {
                ChapterMarkPhase::FadeIn => {
                    mark.alpha = (mark.timer / ChapterMarkState::FADE_IN_DURATION).min(1.0);
                    if mark.timer >= ChapterMarkState::FADE_IN_DURATION {
                        mark.phase = ChapterMarkPhase::Visible;
                        mark.timer = 0.0;
                        mark.alpha = 1.0;
                    }
                    false
                }
                ChapterMarkPhase::Visible => {
                    mark.alpha = 1.0;
                    if mark.timer >= ChapterMarkState::VISIBLE_DURATION {
                        mark.phase = ChapterMarkPhase::FadeOut;
                        mark.timer = 0.0;
                    }
                    false
                }
                ChapterMarkPhase::FadeOut => {
                    mark.alpha = 1.0 - (mark.timer / ChapterMarkState::FADE_OUT_DURATION).min(1.0);
                    if mark.timer >= ChapterMarkState::FADE_OUT_DURATION {
                        true // 动画完成，需要清除
                    } else {
                        false
                    }
                }
            }
        } else {
            return false;
        };

        if should_clear {
            self.chapter_mark = None;
            false
        } else {
            true
        }
    }

    /// 设置选择界面
    pub fn set_choices(&mut self, choices: Vec<ChoiceItem>, style: Option<String>) {
        self.choices = Some(ChoicesState {
            choices,
            style,
            selected_index: 0,
            hovered_index: None,
        });
    }

    /// 清除选择界面
    pub fn clear_choices(&mut self) {
        self.choices = None;
    }
}
