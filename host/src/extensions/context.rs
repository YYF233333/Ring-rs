//! 扩展执行上下文（受控访问核心系统 + 诊断上报）。

use crate::app::CoreSystems;
use crate::manifest::Manifest;

/// 扩展诊断级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Info,
    Warn,
    Error,
}

/// 单条扩展诊断记录。
#[derive(Debug, Clone)]
pub struct ExtensionDiagnostic {
    pub level: DiagnosticLevel,
    pub capability_id: String,
    pub extension_name: String,
    pub message: String,
}

/// 扩展执行上下文。
pub struct EngineContext<'a> {
    core: &'a mut CoreSystems,
    manifest: &'a Manifest,
    diagnostics: Vec<ExtensionDiagnostic>,
}

impl<'a> EngineContext<'a> {
    pub fn new(core: &'a mut CoreSystems, manifest: &'a Manifest) -> Self {
        Self {
            core,
            manifest,
            diagnostics: Vec::new(),
        }
    }

    /// 获取可变核心系统引用（扩展需通过该入口访问底层能力）。
    pub fn core_mut(&mut self) -> &mut CoreSystems {
        self.core
    }

    /// 只读资源清单。
    pub fn manifest(&self) -> &Manifest {
        self.manifest
    }

    pub fn emit_info(&mut self, capability_id: &str, extension_name: &str, message: &str) {
        self.push(
            DiagnosticLevel::Info,
            capability_id,
            extension_name,
            message,
        );
    }

    pub fn emit_warn(&mut self, capability_id: &str, extension_name: &str, message: &str) {
        self.push(
            DiagnosticLevel::Warn,
            capability_id,
            extension_name,
            message,
        );
    }

    pub fn emit_error(&mut self, capability_id: &str, extension_name: &str, message: &str) {
        self.push(
            DiagnosticLevel::Error,
            capability_id,
            extension_name,
            message,
        );
    }

    pub fn take_diagnostics(&mut self) -> Vec<ExtensionDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    fn push(
        &mut self,
        level: DiagnosticLevel,
        capability_id: &str,
        extension_name: &str,
        message: &str,
    ) {
        self.diagnostics.push(ExtensionDiagnostic {
            level,
            capability_id: capability_id.to_string(),
            extension_name: extension_name.to_string(),
            message: message.to_string(),
        });
    }
}
