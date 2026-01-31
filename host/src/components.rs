//! 组件定义模块

use bevy::prelude::*;
use vn_runtime::command::Position;

/// 背景组件
#[derive(Component)]
pub struct Background;

/// 角色立绘组件
#[derive(Component)]
pub struct Character {
    /// 角色别名（用于 hide 指令）
    pub alias: String,
    /// 位置
    pub position: Position,
}

/// 对话框 UI 根节点
#[derive(Component)]
pub struct DialogueBox;

/// 说话者名字文本
#[derive(Component)]
pub struct SpeakerText;

/// 对话内容文本
#[derive(Component)]
pub struct ContentText;

/// 选择按钮
#[derive(Component)]
pub struct ChoiceButton {
    /// 选项索引
    pub index: usize,
}

/// 选择容器
#[derive(Component)]
pub struct ChoiceContainer;

/// 主摄像机标记
#[derive(Component)]
pub struct MainCamera;

