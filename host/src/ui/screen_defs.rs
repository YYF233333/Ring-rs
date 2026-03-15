//! # 界面行为定义模块
//!
//! 从 `ui/screens.json` 加载声明式界面行为配置（按钮列表、动作映射、可见性条件、背景切换），
//! 使新项目无需修改引擎源码即可自定义 UI 行为。
//!
//! 缺失配置时回退到 [`ScreenDefinitions::default()`]，等价于引擎当前硬编码行为。

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
pub struct TitleScreenDef {
    #[serde(default = "defaults::title_background")]
    pub background: Vec<ConditionalAsset>,
    #[serde(default = "defaults::title_overlay")]
    pub overlay: Option<String>,
    #[serde(default = "defaults::title_buttons")]
    pub buttons: Vec<ButtonDef>,
}

impl Default for TitleScreenDef {
    fn default() -> Self {
        Self {
            background: defaults::title_background(),
            overlay: defaults::title_overlay(),
            buttons: defaults::title_buttons(),
        }
    }
}

/// 纯按钮列表定义（ingame_menu / quick_menu）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ButtonListDef {
    #[serde(default)]
    pub buttons: Vec<ButtonDef>,
}

/// 游戏菜单定义（左导航 + 右内容）
#[derive(Debug, Clone, Deserialize)]
pub struct GameMenuDef {
    #[serde(default = "defaults::game_menu_background")]
    pub background: Vec<ConditionalAsset>,
    #[serde(default = "defaults::game_menu_overlay")]
    pub overlay: Option<String>,
    #[serde(default = "defaults::game_menu_nav_buttons")]
    pub nav_buttons: Vec<ButtonDef>,
    #[serde(default = "defaults::game_menu_return_button")]
    pub return_button: ButtonDef,
}

impl Default for GameMenuDef {
    fn default() -> Self {
        Self {
            background: defaults::game_menu_background(),
            overlay: defaults::game_menu_overlay(),
            nav_buttons: defaults::game_menu_nav_buttons(),
            return_button: defaults::game_menu_return_button(),
        }
    }
}

/// 所有界面的行为定义
#[derive(Debug, Clone, Deserialize)]
pub struct ScreenDefinitions {
    #[serde(default)]
    pub title: TitleScreenDef,
    #[serde(default = "defaults::ingame_menu")]
    pub ingame_menu: ButtonListDef,
    #[serde(default = "defaults::quick_menu")]
    pub quick_menu: ButtonListDef,
    #[serde(default)]
    pub game_menu: GameMenuDef,
}

impl Default for ScreenDefinitions {
    fn default() -> Self {
        Self {
            title: TitleScreenDef::default(),
            ingame_menu: defaults::ingame_menu(),
            quick_menu: defaults::quick_menu(),
            game_menu: GameMenuDef::default(),
        }
    }
}

impl ScreenDefinitions {
    /// 从 `ResourceManager` 加载界面行为配置。
    ///
    /// 尝试读取 `ui/screens.json`，失败或缺失时回退到默认值。
    pub fn load(resource_manager: &ResourceManager) -> Self {
        let path = LogicalPath::new("ui/screens.json");
        match resource_manager.read_text_optional(&path) {
            Some(content) => match serde_json::from_str::<Self>(&content) {
                Ok(defs) => {
                    tracing::info!("Screen definitions loaded from ui/screens.json");
                    defs
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to parse ui/screens.json, using defaults");
                    Self::default()
                }
            },
            None => {
                tracing::info!("No ui/screens.json found, using default screen definitions");
                Self::default()
            }
        }
    }
}

// ─── Defaults (等价于当前硬编码行为) ──────────────────────────────────────────

mod defaults {
    use super::*;

    pub fn title_background() -> Vec<ConditionalAsset> {
        vec![
            ConditionalAsset {
                when: Some(ConditionDef::PersistentVar("complete_summer".into())),
                asset: "main_winter".into(),
            },
            ConditionalAsset {
                when: None,
                asset: "main_summer".into(),
            },
        ]
    }

    pub fn title_overlay() -> Option<String> {
        Some("main_menu_overlay".into())
    }

    pub fn title_buttons() -> Vec<ButtonDef> {
        vec![
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
        ]
    }

    pub fn ingame_menu() -> ButtonListDef {
        ButtonListDef {
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
        }
    }

    pub fn quick_menu() -> ButtonListDef {
        ButtonListDef {
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
        }
    }

    pub fn game_menu_background() -> Vec<ConditionalAsset> {
        vec![
            ConditionalAsset {
                when: Some(ConditionDef::PersistentVar("complete_summer".into())),
                asset: "main_winter".into(),
            },
            ConditionalAsset {
                when: None,
                asset: "game_menu_bg".into(),
            },
        ]
    }

    pub fn game_menu_overlay() -> Option<String> {
        Some("game_menu_overlay".into())
    }

    pub fn game_menu_nav_buttons() -> Vec<ButtonDef> {
        vec![
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
        ]
    }

    pub fn game_menu_return_button() -> ButtonDef {
        ButtonDef {
            label: "返回".into(),
            action: ActionDef::ReturnToGame,
            visible: None,
            confirm: None,
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_parse_has_continue() {
        let cond: ConditionDef = serde_json::from_str(r#""$has_continue""#).unwrap();
        assert_eq!(cond, ConditionDef::HasContinue);
    }

    #[test]
    fn condition_parse_persistent_var() {
        let cond: ConditionDef = serde_json::from_str(r#""$persistent.complete_summer""#).unwrap();
        assert_eq!(cond, ConditionDef::PersistentVar("complete_summer".into()));
    }

    #[test]
    fn condition_parse_not_persistent_var() {
        let cond: ConditionDef = serde_json::from_str(r#""!$persistent.complete_summer""#).unwrap();
        assert_eq!(
            cond,
            ConditionDef::NotPersistentVar("complete_summer".into())
        );
    }

    #[test]
    fn condition_parse_true() {
        let cond: ConditionDef = serde_json::from_str(r#""true""#).unwrap();
        assert_eq!(cond, ConditionDef::Always);
    }

    #[test]
    fn action_parse_simple_string() {
        let action: ActionDef = serde_json::from_str(r#""start_game""#).unwrap();
        assert_eq!(action, ActionDef::StartGame);
    }

    #[test]
    fn action_parse_start_at_label() {
        let action: ActionDef = serde_json::from_str(r#"{"start_at_label": "Winter"}"#).unwrap();
        assert_eq!(action, ActionDef::StartAtLabel("Winter".into()));
    }

    #[test]
    fn action_parse_all_string_variants() {
        let cases = [
            ("start_game", ActionDef::StartGame),
            ("continue_game", ActionDef::ContinueGame),
            ("open_load", ActionDef::OpenLoad),
            ("open_save", ActionDef::OpenSave),
            ("navigate_settings", ActionDef::NavigateSettings),
            ("navigate_history", ActionDef::NavigateHistory),
            ("quick_save", ActionDef::QuickSave),
            ("quick_load", ActionDef::QuickLoad),
            ("toggle_skip", ActionDef::ToggleSkip),
            ("toggle_auto", ActionDef::ToggleAuto),
            ("go_back", ActionDef::GoBack),
            ("return_to_title", ActionDef::ReturnToTitle),
            ("return_to_game", ActionDef::ReturnToGame),
            ("exit", ActionDef::Exit),
        ];
        for (input, expected) in cases {
            let json = format!("\"{input}\"");
            let parsed: ActionDef = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, expected, "failed for {input}");
        }
    }

    #[test]
    fn action_parse_unknown_variant_errors() {
        let result = serde_json::from_str::<ActionDef>(r#""unknown_action""#);
        assert!(result.is_err());
    }

    #[test]
    fn button_def_full() {
        let json = r#"{
            "label": "冬篇",
            "action": {"start_at_label": "Winter"},
            "visible": "$persistent.complete_summer",
            "confirm": "确定？"
        }"#;
        let btn: ButtonDef = serde_json::from_str(json).unwrap();
        assert_eq!(btn.label, "冬篇");
        assert_eq!(btn.action, ActionDef::StartAtLabel("Winter".into()));
        assert_eq!(
            btn.visible,
            Some(ConditionDef::PersistentVar("complete_summer".into()))
        );
        assert_eq!(btn.confirm, Some("确定？".into()));
    }

    #[test]
    fn button_def_minimal() {
        let json = r#"{"label": "开始", "action": "start_game"}"#;
        let btn: ButtonDef = serde_json::from_str(json).unwrap();
        assert_eq!(btn.label, "开始");
        assert_eq!(btn.action, ActionDef::StartGame);
        assert!(btn.visible.is_none());
        assert!(btn.confirm.is_none());
    }

    #[test]
    fn screen_definitions_partial_override() {
        let json = r#"{
            "title": {
                "buttons": [
                    {"label": "Play", "action": "start_game"}
                ]
            }
        }"#;
        let defs: ScreenDefinitions = serde_json::from_str(json).unwrap();
        assert_eq!(defs.title.buttons.len(), 1);
        assert_eq!(defs.title.buttons[0].label, "Play");
        // Other screens keep defaults
        assert_eq!(defs.ingame_menu.buttons.len(), 7);
        assert_eq!(defs.quick_menu.buttons.len(), 7);
        assert_eq!(defs.game_menu.nav_buttons.len(), 6);
    }

    #[test]
    fn screen_definitions_default_matches_hardcoded() {
        let defs = ScreenDefinitions::default();

        // Title: 6 buttons
        assert_eq!(defs.title.buttons.len(), 6);
        assert_eq!(defs.title.buttons[0].label, "开始游戏");
        assert_eq!(defs.title.buttons[0].action, ActionDef::StartGame);
        assert_eq!(defs.title.buttons[1].label, "冬篇");
        assert_eq!(
            defs.title.buttons[1].action,
            ActionDef::StartAtLabel("Winter".into())
        );
        assert_eq!(defs.title.buttons[5].label, "退出");
        assert_eq!(defs.title.buttons[5].confirm, Some("确定退出游戏？".into()));

        // Title background: winter (conditional) + summer (fallback)
        assert_eq!(defs.title.background.len(), 2);
        assert_eq!(defs.title.background[0].asset, "main_winter");
        assert_eq!(defs.title.background[1].asset, "main_summer");

        // Ingame menu: 7 buttons
        assert_eq!(defs.ingame_menu.buttons.len(), 7);
        assert_eq!(defs.ingame_menu.buttons[0].label, "继续");

        // Quick menu: 7 buttons
        assert_eq!(defs.quick_menu.buttons.len(), 7);
        assert_eq!(defs.quick_menu.buttons[0].label, "历史");

        // Game menu: 6 nav + return
        assert_eq!(defs.game_menu.nav_buttons.len(), 6);
        assert_eq!(defs.game_menu.return_button.label, "返回");
    }

    #[test]
    fn conditional_asset_resolve_fallback() {
        let assets = vec![
            ConditionalAsset {
                when: Some(ConditionDef::PersistentVar("complete_summer".into())),
                asset: "main_winter".into(),
            },
            ConditionalAsset {
                when: None,
                asset: "main_summer".into(),
            },
        ];

        let store = PersistentStore::empty();
        let ctx = ConditionContext {
            has_continue: false,
            persistent: &store,
        };
        assert_eq!(
            ConditionalAsset::resolve(&assets, &ctx),
            Some("main_summer")
        );
    }

    #[test]
    fn conditional_asset_resolve_match() {
        let assets = vec![
            ConditionalAsset {
                when: Some(ConditionDef::PersistentVar("complete_summer".into())),
                asset: "main_winter".into(),
            },
            ConditionalAsset {
                when: None,
                asset: "main_summer".into(),
            },
        ];

        let mut store = PersistentStore::empty();
        store
            .variables
            .insert("complete_summer".into(), VarValue::Bool(true));
        let ctx = ConditionContext {
            has_continue: false,
            persistent: &store,
        };
        assert_eq!(
            ConditionalAsset::resolve(&assets, &ctx),
            Some("main_winter")
        );
    }

    #[test]
    fn condition_evaluate_has_continue() {
        let store = PersistentStore::empty();
        let ctx_true = ConditionContext {
            has_continue: true,
            persistent: &store,
        };
        let ctx_false = ConditionContext {
            has_continue: false,
            persistent: &store,
        };
        assert!(ConditionDef::HasContinue.evaluate(&ctx_true));
        assert!(!ConditionDef::HasContinue.evaluate(&ctx_false));
    }

    #[test]
    fn condition_evaluate_persistent_var() {
        let mut store = PersistentStore::empty();
        store
            .variables
            .insert("complete_summer".into(), VarValue::Bool(true));

        let ctx = ConditionContext {
            has_continue: false,
            persistent: &store,
        };

        assert!(ConditionDef::PersistentVar("complete_summer".into()).evaluate(&ctx));
        assert!(!ConditionDef::PersistentVar("nonexistent".into()).evaluate(&ctx));
        assert!(!ConditionDef::NotPersistentVar("complete_summer".into()).evaluate(&ctx));
        assert!(ConditionDef::NotPersistentVar("nonexistent".into()).evaluate(&ctx));
    }
}
