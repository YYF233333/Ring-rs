//! # 界面行为定义模块
//!
//! 从 `ui/screens.json` 加载声明式界面行为配置（按钮列表、动作映射、可见性条件、背景切换），
//! 使新项目无需修改引擎源码即可自定义 UI 行为。
//!
//! 配置文件必须存在且字段完整，否则启动报错。

use std::fmt;

use serde::Deserialize;
use serde::de::{self, MapAccess, Visitor};
use vn_runtime::state::VarValue;

use crate::app::persistent::PersistentStore;
use crate::resources::{LogicalPath, ResourceManager};

// ─── ConditionDef ─────────────────────────────────────────────────────────────

/// 可见性/背景选择条件
///
/// 极简单变量布尔判断，覆盖当前所有已知条件需求。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConditionDef {
    /// 始终为 true
    Always,
    /// 存在可继续的存档
    HasContinue,
    /// 持久化变量为 truthy
    PersistentVar(String),
    /// 持久化变量不为 truthy
    NotPersistentVar(String),
}

impl<'de> Deserialize<'de> for ConditionDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(parse_condition(&s))
    }
}

fn parse_condition(s: &str) -> ConditionDef {
    match s {
        "true" | "" => ConditionDef::Always,
        "$has_continue" => ConditionDef::HasContinue,
        _ if s.starts_with("!$persistent.") => {
            ConditionDef::NotPersistentVar(s["!$persistent.".len()..].to_string())
        }
        _ if s.starts_with("$persistent.") => {
            ConditionDef::PersistentVar(s["$persistent.".len()..].to_string())
        }
        other => {
            tracing::warn!(
                condition = other,
                "Unknown condition syntax, treating as Always"
            );
            ConditionDef::Always
        }
    }
}

// ─── Condition Evaluation ─────────────────────────────────────────────────────

/// 条件求值上下文（在渲染循环开始时一次性构造）
pub struct ConditionContext<'a> {
    pub has_continue: bool,
    pub persistent: &'a PersistentStore,
}

impl ConditionDef {
    /// 在给定上下文中求值
    pub fn evaluate(&self, ctx: &ConditionContext<'_>) -> bool {
        match self {
            ConditionDef::Always => true,
            ConditionDef::HasContinue => ctx.has_continue,
            ConditionDef::PersistentVar(key) => is_var_truthy(ctx.persistent, key),
            ConditionDef::NotPersistentVar(key) => !is_var_truthy(ctx.persistent, key),
        }
    }
}

fn is_var_truthy(store: &PersistentStore, key: &str) -> bool {
    store
        .variables
        .get(key)
        .is_some_and(|v| matches!(v, VarValue::Bool(true)))
}

// ─── ActionDef ────────────────────────────────────────────────────────────────

/// 声明式动作定义
///
/// 编译期已知的动作词汇表，由 binary crate 转换为 `EguiAction`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionDef {
    StartGame,
    ContinueGame,
    OpenLoad,
    OpenSave,
    NavigateSettings,
    NavigateHistory,
    /// 替换当前模式为 Settings（不压栈，用于游戏菜单同级切换）
    ReplaceSettings,
    /// 替换当前模式为 History（不压栈，用于游戏菜单同级切换）
    ReplaceHistory,
    QuickSave,
    QuickLoad,
    ToggleSkip,
    ToggleAuto,
    GoBack,
    ReturnToTitle,
    ReturnToGame,
    Exit,
    /// 从指定标签开始新游戏（泛化 `StartWinter`）
    StartAtLabel(String),
}

impl<'de> Deserialize<'de> for ActionDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ActionDefVisitor;

        impl<'de> Visitor<'de> for ActionDefVisitor {
            type Value = ActionDef;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a string action ID or an object like {\"start_at_label\": \"X\"}")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<ActionDef, E> {
                match v {
                    "start_game" => Ok(ActionDef::StartGame),
                    "continue_game" => Ok(ActionDef::ContinueGame),
                    "open_load" => Ok(ActionDef::OpenLoad),
                    "open_save" => Ok(ActionDef::OpenSave),
                    "navigate_settings" => Ok(ActionDef::NavigateSettings),
                    "navigate_history" => Ok(ActionDef::NavigateHistory),
                    "replace_settings" => Ok(ActionDef::ReplaceSettings),
                    "replace_history" => Ok(ActionDef::ReplaceHistory),
                    "quick_save" => Ok(ActionDef::QuickSave),
                    "quick_load" => Ok(ActionDef::QuickLoad),
                    "toggle_skip" => Ok(ActionDef::ToggleSkip),
                    "toggle_auto" => Ok(ActionDef::ToggleAuto),
                    "go_back" => Ok(ActionDef::GoBack),
                    "return_to_title" => Ok(ActionDef::ReturnToTitle),
                    "return_to_game" => Ok(ActionDef::ReturnToGame),
                    "exit" => Ok(ActionDef::Exit),
                    other => Err(de::Error::unknown_variant(
                        other,
                        &[
                            "start_game",
                            "continue_game",
                            "open_load",
                            "open_save",
                            "navigate_settings",
                            "navigate_history",
                            "replace_settings",
                            "replace_history",
                            "quick_save",
                            "quick_load",
                            "toggle_skip",
                            "toggle_auto",
                            "go_back",
                            "return_to_title",
                            "return_to_game",
                            "exit",
                        ],
                    )),
                }
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<ActionDef, M::Error> {
                let mut start_at_label: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "start_at_label" => {
                            start_at_label = Some(map.next_value()?);
                        }
                        other => {
                            return Err(de::Error::unknown_field(other, &["start_at_label"]));
                        }
                    }
                }
                start_at_label
                    .map(ActionDef::StartAtLabel)
                    .ok_or_else(|| de::Error::missing_field("start_at_label"))
            }
        }

        deserializer.deserialize_any(ActionDefVisitor)
    }
}

// ─── ButtonDef ────────────────────────────────────────────────────────────────

/// 按钮定义
#[derive(Debug, Clone, Deserialize)]
pub struct ButtonDef {
    /// 按钮文案
    pub label: String,
    /// 点击动作
    pub action: ActionDef,
    /// 可见性条件（缺失视为 Always）
    #[serde(default)]
    pub visible: Option<ConditionDef>,
    /// 确认弹窗文案（非 None 时点击先弹确认）
    #[serde(default)]
    pub confirm: Option<String>,
}

// ─── ConditionalAsset ─────────────────────────────────────────────────────────

/// 条件化资源引用（用于背景切换）
#[derive(Debug, Clone, Deserialize)]
pub struct ConditionalAsset {
    /// 条件（缺失视为 Always，即兜底资源）
    #[serde(default)]
    pub when: Option<ConditionDef>,
    /// 资源 key
    pub asset: String,
}

impl ConditionalAsset {
    /// 从条件资源列表中选出第一个满足条件的资源 key
    pub fn resolve<'a>(
        assets: &'a [ConditionalAsset],
        ctx: &ConditionContext<'_>,
    ) -> Option<&'a str> {
        assets.iter().find_map(|ca| {
            let passes = ca.when.as_ref().is_none_or(|cond| cond.evaluate(ctx));
            passes.then_some(ca.asset.as_str())
        })
    }
}

// ─── Screen Definitions ───────────────────────────────────────────────────────

/// 标题页定义
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TitleScreenDef {
    pub background: Vec<ConditionalAsset>,
    pub overlay: Option<String>,
    pub buttons: Vec<ButtonDef>,
}

impl Default for TitleScreenDef {
    fn default() -> Self {
        Self {
            background: vec![
                ConditionalAsset {
                    when: Some(ConditionDef::PersistentVar("complete_summer".into())),
                    asset: "main_winter".into(),
                },
                ConditionalAsset {
                    when: None,
                    asset: "main_summer".into(),
                },
            ],
            overlay: Some("main_menu_overlay".into()),
            buttons: vec![
                ButtonDef {
                    label: "开始游戏".into(),
                    action: ActionDef::StartGame,
                    visible: None,
                    confirm: None,
                },
                ButtonDef {
                    label: "冬篇".into(),
                    action: ActionDef::StartAtLabel("Winter".into()),
                    visible: Some(ConditionDef::PersistentVar("complete_summer".into())),
                    confirm: None,
                },
                ButtonDef {
                    label: "继续游戏".into(),
                    action: ActionDef::ContinueGame,
                    visible: Some(ConditionDef::HasContinue),
                    confirm: None,
                },
                ButtonDef {
                    label: "读取游戏".into(),
                    action: ActionDef::OpenLoad,
                    visible: None,
                    confirm: None,
                },
                ButtonDef {
                    label: "设置".into(),
                    action: ActionDef::NavigateSettings,
                    visible: None,
                    confirm: None,
                },
                ButtonDef {
                    label: "退出".into(),
                    action: ActionDef::Exit,
                    visible: None,
                    confirm: Some("确定退出游戏？".into()),
                },
            ],
        }
    }
}

/// 纯按钮列表定义（ingame_menu / quick_menu）
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ButtonListDef {
    pub buttons: Vec<ButtonDef>,
}

/// 游戏菜单定义（左导航 + 右内容）
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameMenuDef {
    pub background: Vec<ConditionalAsset>,
    pub overlay: Option<String>,
    pub nav_buttons: Vec<ButtonDef>,
    pub return_button: ButtonDef,
}

impl Default for GameMenuDef {
    fn default() -> Self {
        Self {
            background: vec![
                ConditionalAsset {
                    when: Some(ConditionDef::PersistentVar("complete_summer".into())),
                    asset: "main_winter".into(),
                },
                ConditionalAsset {
                    when: None,
                    asset: "game_menu_bg".into(),
                },
            ],
            overlay: Some("game_menu_overlay".into()),
            nav_buttons: vec![
                ButtonDef {
                    label: "历史".into(),
                    action: ActionDef::ReplaceHistory,
                    visible: None,
                    confirm: None,
                },
                ButtonDef {
                    label: "保存".into(),
                    action: ActionDef::OpenSave,
                    visible: None,
                    confirm: None,
                },
                ButtonDef {
                    label: "读取".into(),
                    action: ActionDef::OpenLoad,
                    visible: None,
                    confirm: None,
                },
                ButtonDef {
                    label: "设置".into(),
                    action: ActionDef::ReplaceSettings,
                    visible: None,
                    confirm: None,
                },
                ButtonDef {
                    label: "返回标题".into(),
                    action: ActionDef::ReturnToTitle,
                    visible: None,
                    confirm: Some("确定返回标题画面？".into()),
                },
                ButtonDef {
                    label: "退出".into(),
                    action: ActionDef::Exit,
                    visible: None,
                    confirm: Some("确定退出游戏？".into()),
                },
            ],
            return_button: ButtonDef {
                label: "返回".into(),
                action: ActionDef::ReturnToGame,
                visible: None,
                confirm: None,
            },
        }
    }
}

/// 所有界面的行为定义
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenDefinitions {
    pub title: TitleScreenDef,
    pub ingame_menu: ButtonListDef,
    pub quick_menu: ButtonListDef,
    pub game_menu: GameMenuDef,
}

impl Default for ScreenDefinitions {
    fn default() -> Self {
        Self {
            title: TitleScreenDef::default(),
            ingame_menu: ButtonListDef {
                buttons: vec![
                    ButtonDef {
                        label: "继续".into(),
                        action: ActionDef::GoBack,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "保存".into(),
                        action: ActionDef::OpenSave,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "读取".into(),
                        action: ActionDef::OpenLoad,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "设置".into(),
                        action: ActionDef::NavigateSettings,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "历史".into(),
                        action: ActionDef::NavigateHistory,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "返回标题".into(),
                        action: ActionDef::ReturnToTitle,
                        visible: None,
                        confirm: Some("确定返回标题画面？".into()),
                    },
                    ButtonDef {
                        label: "退出".into(),
                        action: ActionDef::Exit,
                        visible: None,
                        confirm: Some("确定退出游戏？".into()),
                    },
                ],
            },
            quick_menu: ButtonListDef {
                buttons: vec![
                    ButtonDef {
                        label: "历史".into(),
                        action: ActionDef::NavigateHistory,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "快进".into(),
                        action: ActionDef::ToggleSkip,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "自动".into(),
                        action: ActionDef::ToggleAuto,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "保存".into(),
                        action: ActionDef::OpenSave,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "快存".into(),
                        action: ActionDef::QuickSave,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "快读".into(),
                        action: ActionDef::QuickLoad,
                        visible: None,
                        confirm: None,
                    },
                    ButtonDef {
                        label: "设置".into(),
                        action: ActionDef::NavigateSettings,
                        visible: None,
                        confirm: None,
                    },
                ],
            },
            game_menu: GameMenuDef::default(),
        }
    }
}

/// 界面行为配置加载错误
#[derive(Debug)]
pub enum ScreenDefsError {
    /// 配置文件缺失或读取失败
    NotFound(String),
    /// JSON 解析失败
    ParseFailed(String),
}

impl std::fmt::Display for ScreenDefsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScreenDefsError::NotFound(msg) => write!(f, "界面配置加载失败: {}", msg),
            ScreenDefsError::ParseFailed(msg) => write!(f, "界面配置解析失败: {}", msg),
        }
    }
}

impl std::error::Error for ScreenDefsError {}

impl ScreenDefinitions {
    /// 从 `ResourceManager` 加载界面行为配置。
    ///
    /// 配置文件 `ui/screens.json` 必须存在且字段完整，否则返回错误。
    pub fn load(resource_manager: &ResourceManager) -> Result<Self, ScreenDefsError> {
        let path = LogicalPath::new("ui/screens.json");
        let content = resource_manager
            .read_text_optional(&path)
            .ok_or_else(|| ScreenDefsError::NotFound("ui/screens.json 不存在".into()))?;

        let defs: Self = serde_json::from_str(&content)
            .map_err(|e| ScreenDefsError::ParseFailed(format!("ui/screens.json: {e}")))?;

        tracing::info!("Screen definitions loaded from ui/screens.json");
        Ok(defs)
    }
}

#[cfg(test)]
mod tests;
