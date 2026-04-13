//! # 界面行为定义模块
//!
//! 从 `ui/screens.json` 加载声明式界面行为配置（按钮列表、动作映射、可见性条件、背景切换），
//! 使新项目无需修改引擎源码即可自定义 UI 行为。
//!
//! 配置文件必须存在且字段完整，否则启动报错。
//!
//! 复用自 `host/src/ui/screen_defs/mod.rs`，移除 egui 依赖。

use std::fmt;

use serde::Deserialize;
use serde::de::{self, MapAccess, Visitor};
use vn_runtime::state::VarValue;

use crate::resources::{LogicalPath, ResourceManager};
use crate::state::PersistentStore;

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
/// 编译期已知的动作词汇表，由 binary crate 映射为应用层操作。
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

/// 纯按钮列表定义（ingame_menu / quick_menu）
#[derive(Debug, Clone, Deserialize)]
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

/// 所有界面的行为定义
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenDefinitions {
    pub title: TitleScreenDef,
    pub ingame_menu: ButtonListDef,
    pub quick_menu: ButtonListDef,
    pub game_menu: GameMenuDef,
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
    /// 测试用默认值（不依赖资源文件）
    #[cfg(test)]
    pub fn default_for_tests() -> Self {
        Self {
            title: TitleScreenDef {
                background: vec![ConditionalAsset {
                    when: None,
                    asset: "main_summer".into(),
                }],
                overlay: Some("main_menu_overlay".into()),
                buttons: vec![ButtonDef {
                    label: "开始游戏".into(),
                    action: ActionDef::StartGame,
                    visible: None,
                    confirm: None,
                }],
            },
            ingame_menu: ButtonListDef {
                buttons: vec![ButtonDef {
                    label: "继续".into(),
                    action: ActionDef::GoBack,
                    visible: None,
                    confirm: None,
                }],
            },
            quick_menu: ButtonListDef { buttons: vec![] },
            game_menu: GameMenuDef {
                background: vec![ConditionalAsset {
                    when: None,
                    asset: "game_menu_bg".into(),
                }],
                overlay: Some("game_menu_overlay".into()),
                nav_buttons: vec![],
                return_button: ButtonDef {
                    label: "返回".into(),
                    action: ActionDef::ReturnToGame,
                    visible: None,
                    confirm: None,
                },
            },
        }
    }

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
mod tests {
    use vn_runtime::state::VarValue;

    use crate::state::PersistentStore;

    use super::*;

    fn empty_persistent() -> PersistentStore {
        PersistentStore::empty()
    }

    fn persistent_with(key: &str, val: VarValue) -> PersistentStore {
        let mut store = PersistentStore::empty();
        store.variables.insert(key.to_string(), val);
        store
    }

    // ── parse_condition ────────────────────────────────────────────────────────

    #[test]
    fn parse_condition_always_variants() {
        assert_eq!(parse_condition("true"), ConditionDef::Always);
        assert_eq!(parse_condition(""), ConditionDef::Always);
    }

    #[test]
    fn parse_condition_has_continue() {
        assert_eq!(parse_condition("$has_continue"), ConditionDef::HasContinue);
    }

    #[test]
    fn parse_condition_persistent_var() {
        assert_eq!(
            parse_condition("$persistent.my_flag"),
            ConditionDef::PersistentVar("my_flag".to_string())
        );
    }

    #[test]
    fn parse_condition_not_persistent_var() {
        assert_eq!(
            parse_condition("!$persistent.my_flag"),
            ConditionDef::NotPersistentVar("my_flag".to_string())
        );
    }

    #[test]
    fn parse_condition_unknown_defaults_to_always() {
        assert_eq!(parse_condition("$unknown_syntax"), ConditionDef::Always);
    }

    // ── ConditionDef::evaluate ────────────────────────────────────────────────

    #[test]
    fn condition_always_is_true_regardless_of_context() {
        let store = empty_persistent();
        let ctx = ConditionContext {
            has_continue: false,
            persistent: &store,
        };
        assert!(ConditionDef::Always.evaluate(&ctx));
    }

    #[test]
    fn condition_has_continue_mirrors_context_flag() {
        let store = empty_persistent();
        let ctx_no = ConditionContext {
            has_continue: false,
            persistent: &store,
        };
        let ctx_yes = ConditionContext {
            has_continue: true,
            persistent: &store,
        };
        assert!(!ConditionDef::HasContinue.evaluate(&ctx_no));
        assert!(ConditionDef::HasContinue.evaluate(&ctx_yes));
    }

    #[test]
    fn condition_persistent_var_true_when_set() {
        let store_set = persistent_with("flag", VarValue::Bool(true));
        let store_unset = empty_persistent();
        let ctx_set = ConditionContext {
            has_continue: false,
            persistent: &store_set,
        };
        let ctx_unset = ConditionContext {
            has_continue: false,
            persistent: &store_unset,
        };
        assert!(ConditionDef::PersistentVar("flag".to_string()).evaluate(&ctx_set));
        assert!(!ConditionDef::PersistentVar("flag".to_string()).evaluate(&ctx_unset));
    }

    #[test]
    fn condition_not_persistent_var_inverts_truthy() {
        let store_set = persistent_with("flag", VarValue::Bool(true));
        let ctx = ConditionContext {
            has_continue: false,
            persistent: &store_set,
        };
        assert!(!ConditionDef::NotPersistentVar("flag".to_string()).evaluate(&ctx));

        let store_unset = empty_persistent();
        let ctx2 = ConditionContext {
            has_continue: false,
            persistent: &store_unset,
        };
        assert!(ConditionDef::NotPersistentVar("flag".to_string()).evaluate(&ctx2));
    }

    // ── ActionDef deserialization ─────────────────────────────────────────────

    #[test]
    fn action_def_deserialize_string_variants() {
        let pairs: &[(&str, ActionDef)] = &[
            (r#""start_game""#, ActionDef::StartGame),
            (r#""continue_game""#, ActionDef::ContinueGame),
            (r#""go_back""#, ActionDef::GoBack),
            (r#""return_to_title""#, ActionDef::ReturnToTitle),
            (r#""exit""#, ActionDef::Exit),
            (r#""quick_save""#, ActionDef::QuickSave),
            (r#""quick_load""#, ActionDef::QuickLoad),
        ];
        for (json, expected) in pairs {
            let got: ActionDef = serde_json::from_str(json).unwrap();
            assert_eq!(got, *expected, "failed for {json}");
        }
    }

    #[test]
    fn action_def_deserialize_start_at_label_object() {
        let json = r#"{"start_at_label": "prologue"}"#;
        let got: ActionDef = serde_json::from_str(json).unwrap();
        assert_eq!(got, ActionDef::StartAtLabel("prologue".to_string()));
    }

    #[test]
    fn action_def_deserialize_unknown_string_errors() {
        let result = serde_json::from_str::<ActionDef>(r#""not_a_real_action""#);
        assert!(result.is_err());
    }

    // ── ConditionalAsset::resolve ──────────────────────────────────────────────

    #[test]
    fn conditional_asset_resolve_first_matching_condition() {
        let store = persistent_with("flag", VarValue::Bool(true));
        let ctx = ConditionContext {
            has_continue: false,
            persistent: &store,
        };
        let assets = vec![
            ConditionalAsset {
                when: Some(ConditionDef::PersistentVar("flag".to_string())),
                asset: "asset_when_set".to_string(),
            },
            ConditionalAsset {
                when: None,
                asset: "fallback".to_string(),
            },
        ];
        assert_eq!(
            ConditionalAsset::resolve(&assets, &ctx),
            Some("asset_when_set")
        );
    }

    #[test]
    fn conditional_asset_resolve_falls_back_when_condition_unmet() {
        let store = empty_persistent();
        let ctx = ConditionContext {
            has_continue: false,
            persistent: &store,
        };
        let assets = vec![
            ConditionalAsset {
                when: Some(ConditionDef::PersistentVar("flag".to_string())),
                asset: "asset_when_set".to_string(),
            },
            ConditionalAsset {
                when: None,
                asset: "fallback".to_string(),
            },
        ];
        assert_eq!(ConditionalAsset::resolve(&assets, &ctx), Some("fallback"));
    }

    #[test]
    fn conditional_asset_resolve_empty_list_returns_none() {
        let store = empty_persistent();
        let ctx = ConditionContext {
            has_continue: false,
            persistent: &store,
        };
        assert_eq!(ConditionalAsset::resolve(&[], &ctx), None);
    }
}
