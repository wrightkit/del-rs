//! Lower the backend-neutral DEL HIR into canonical `workshop-rs` WIR.
//!
//! This module owns the DEL-side lowering policy only. Workshop identities,
//! event shapes, variable/action/value nodes, validation, and emission remain
//! owned by `workshop-rs`.

use crate::diagnostics::{error, Diagnostic, Phase};
use crate::hir::{
    CallTarget, HirArg, HirExprId, HirExprKind, HirFuncId, HirProgram, HirStmt, HirStmtKind,
    HirVarId, LiteralValue, StorageIntent,
};
use crate::project::Project;
use crate::semantic::provider::ExternalBinding;
use crate::span::{FileId, SourceMap, Span};
use crate::syntax::ast::{AssignOp, BinaryOp, UnaryOp};
use std::collections::{HashMap, HashSet};
use workshop_rs::catalog::Catalog;
use workshop_rs::source::{Position, SourceFile, Span as WorkshopSpan};
use workshop_rs::wir;

/// Lower a validated HIR program into canonical Workshop WIR.
///
/// The source map is required because HIR spans use DEL byte offsets while
/// WIR provenance uses 1-based source positions. The returned diagnostics are
/// fail-closed: an unsupported construct never becomes a successful but
/// semantically incomplete WIR node.
pub fn lower_to_wir(hir: &HirProgram, sources: &SourceMap) -> (wir::Program, Vec<Diagnostic>) {
    Lowerer::new(hir, sources).run()
}

/// Convenience entry point for callers that still own the checked semantic
/// program. HIR is lowered first, then lowered into WIR with the same project
/// source registry.
pub fn lower_project_to_wir(
    semantic: &crate::semantic::SemanticProgram,
) -> (wir::Program, Vec<Diagnostic>) {
    let (hir, mut diagnostics) = crate::hir::lower::lower(semantic);
    let (program, mut lowering) = lower_to_wir(&hir, &semantic.project.sources);
    diagnostics.append(&mut lowering);
    (program, diagnostics)
}

/// Lower a checked project directly. This preserves the public project
/// boundary without making the WIR backend depend on the semantic provider.
pub fn lower_project(
    project: &Project,
    provider: &dyn crate::semantic::provider::WorkshopProvider,
) -> (wir::Program, Vec<Diagnostic>) {
    let semantic = crate::semantic::check_project(project, provider);
    lower_project_to_wir(&semantic)
}

struct Lowerer<'a> {
    hir: &'a HirProgram,
    sources: &'a SourceMap,
    out: wir::Program,
    global_vars: HashMap<HirVarId, wir::GlobalVarId>,
    player_vars: HashMap<HirVarId, wir::PlayerVarId>,
    subroutines: HashMap<HirFuncId, wir::SubroutineId>,
    diagnostics: Vec<Diagnostic>,
    used_global_indices: HashSet<u32>,
    used_player_indices: HashSet<u32>,
}

impl<'a> Lowerer<'a> {
    fn new(hir: &'a HirProgram, sources: &'a SourceMap) -> Self {
        let mut out = wir::Program::default();
        for source in sources.files() {
            out.files
                .push(SourceFile::new(source.name.display().to_string()));
        }
        Self {
            hir,
            sources,
            out,
            global_vars: HashMap::new(),
            player_vars: HashMap::new(),
            subroutines: HashMap::new(),
            diagnostics: Vec::new(),
            used_global_indices: HashSet::new(),
            used_player_indices: HashSet::new(),
        }
    }

    fn run(mut self) -> (wir::Program, Vec<Diagnostic>) {
        self.allocate_variables();
        self.allocate_subroutines();
        self.lower_initializers();
        for rule in &self.hir.rules {
            self.lower_rule(rule);
        }
        for (fid, func) in self.hir.funcs.iter().enumerate() {
            if func.kind == crate::hir::FuncKind::Subroutine {
                self.lower_subroutine(fid as HirFuncId);
            }
        }
        self.validate_output();
        (self.out, self.diagnostics)
    }

    fn allocate_variables(&mut self) {
        for (id, var) in self.hir.vars.iter().enumerate() {
            let id = id as HirVarId;
            match var.storage {
                StorageIntent::Global => {
                    let index = self.allocate_index(var.explicit_id, false, var.span);
                    let wir_id = self.out.global_variables.push(wir::WorkshopVariable {
                        name: var.name.clone(),
                        index,
                        span: self.ws_span(var.span),
                        name_span: self.ws_span(var.span),
                    });
                    self.global_vars.insert(id, wir_id);
                }
                StorageIntent::Player => {
                    let index = self.allocate_index(var.explicit_id, true, var.span);
                    let wir_id = self.out.player_variables.push(wir::WorkshopVariable {
                        name: var.name.clone(),
                        index,
                        span: self.ws_span(var.span),
                        name_span: self.ws_span(var.span),
                    });
                    self.player_vars.insert(id, wir_id);
                }
                StorageIntent::Local
                | StorageIntent::Member
                | StorageIntent::StaticMember
                | StorageIntent::Parameter
                | StorageIntent::External => {}
            }
        }
        for reservation in &self.hir.reservations {
            for name in &reservation.names {
                match reservation.storage {
                    StorageIntent::Global => {
                        let index = self.allocate_index(None, false, reservation.span);
                        self.out.global_variables.push(wir::WorkshopVariable {
                            name: name.clone(),
                            index,
                            span: self.ws_span(reservation.span),
                            name_span: self.ws_span(reservation.span),
                        });
                    }
                    StorageIntent::Player => {
                        let index = self.allocate_index(None, true, reservation.span);
                        self.out.player_variables.push(wir::WorkshopVariable {
                            name: name.clone(),
                            index,
                            span: self.ws_span(reservation.span),
                            name_span: self.ws_span(reservation.span),
                        });
                    }
                    _ => self.unsupported(reservation.span, "invalid variable reservation storage"),
                }
            }
        }
    }

    fn allocate_index(&mut self, explicit: Option<u32>, player: bool, span: Span) -> u32 {
        let used = if player {
            &mut self.used_player_indices
        } else {
            &mut self.used_global_indices
        };
        if let Some(index) = explicit {
            if !used.insert(index) {
                self.unsupported(
                    span,
                    format!("duplicate explicit Workshop variable index {index}"),
                );
            }
            return index;
        }
        let mut index = 0;
        while used.contains(&index) {
            index += 1;
        }
        used.insert(index);
        index
    }

    fn allocate_subroutines(&mut self) {
        for (index, func) in self.hir.funcs.iter().enumerate() {
            if func.kind != crate::hir::FuncKind::Subroutine {
                continue;
            }
            let id = self.out.subroutines.push(wir::WorkshopSubroutine {
                name: func.name.clone(),
                index: self.out.subroutines.len() as u32,
                span: self.ws_span(func.span),
                name_span: self.ws_span(func.span),
            });
            self.subroutines.insert(index as HirFuncId, id);
        }
    }

    fn lower_initializers(&mut self) {
        let mut global_actions = Vec::new();
        let mut player_actions = Vec::new();
        for stmt in &self.hir.top {
            let HirStmtKind::VarDecl { var, init } = stmt.kind else {
                self.unsupported(
                    stmt.span,
                    "top-level initializer is not a variable declaration",
                );
                continue;
            };
            let Some(init) = init else { continue };
            let Ok(value) = self.lower_value(init) else {
                continue;
            };
            let Some(hir_var) = self.hir.vars.get(var as usize) else {
                self.unsupported(
                    stmt.span,
                    format!("initializer references unknown HIR variable {var}"),
                );
                continue;
            };
            match hir_var.storage {
                StorageIntent::Global => {
                    if let Some(variable) = self.global_vars.get(&var).copied() {
                        global_actions.push(self.out.actions.push(
                            wir::Action::SetGlobalVariable {
                                variable,
                                value,
                                span: self.ws_span(stmt.span),
                                target_span: self.ws_span(hir_var.span),
                            },
                        ));
                    }
                }
                StorageIntent::Player => {
                    if let Some(variable) = self.player_vars.get(&var).copied() {
                        let player = self.out.values.push(wir::ValueNode::new(
                            wir::Value::EventPlayer,
                            self.ws_span(stmt.span),
                        ));
                        player_actions.push(self.out.actions.push(
                            wir::Action::SetPlayerVariable {
                                player,
                                variable,
                                value,
                                span: self.ws_span(stmt.span),
                                target_span: self.ws_span(hir_var.span),
                            },
                        ));
                    }
                }
                _ => self.unsupported(
                    stmt.span,
                    "top-level initializer targets a non-Workshop variable",
                ),
            }
        }
        if !global_actions.is_empty() {
            self.out.rules.push(wir::Rule {
                name: "Initialize Global Variables".to_string(),
                span: None,
                name_span: None,
                disabled: false,
                event: wir::Event::Global,
                conditions: Vec::new(),
                actions: global_actions,
            });
        }
        if !player_actions.is_empty() {
            self.out.rules.push(wir::Rule {
                name: "Initialize Player Variables".to_string(),
                span: None,
                name_span: None,
                disabled: false,
                event: wir::Event::EachPlayer,
                conditions: Vec::new(),
                actions: player_actions,
            });
        }
    }

    fn lower_rule(&mut self, rule: &crate::hir::HirRule) {
        let diagnostic_count = self.diagnostics.len();
        let Some(event) = rule.event.and_then(|id| self.lower_event(id)) else {
            if rule.event.is_none() {
                self.unsupported(rule.span, "rule has no canonical Workshop event");
            }
            return;
        };
        let mut conditions = Vec::new();
        for condition in &rule.conditions {
            if let Ok(value) = self.lower_value(condition.expr) {
                conditions.push(value);
            }
        }
        let actions = self.lower_actions(&rule.body);
        if self.has_new_errors(diagnostic_count) {
            return;
        }
        self.out.rules.push(wir::Rule {
            name: rule.name.clone().unwrap_or_default(),
            span: self.ws_span(rule.span),
            name_span: rule.name_span.and_then(|span| self.ws_span(span)),
            disabled: rule.disabled,
            event,
            conditions,
            actions,
        });
    }

    fn lower_subroutine(&mut self, fid: HirFuncId) {
        let Some(func) = self.hir.funcs.get(fid as usize) else {
            return;
        };
        let Some(body) = func.body.as_ref() else {
            self.unsupported(func.span, format!("subroutine '{}' has no body", func.name));
            return;
        };
        let diagnostic_count = self.diagnostics.len();
        let actions = self.lower_actions(body);
        if self.has_new_errors(diagnostic_count) {
            return;
        }
        let Some(subroutine) = self.subroutines.get(&fid).copied() else {
            return;
        };
        self.out.rules.push(wir::Rule {
            name: func.name.clone(),
            span: self.ws_span(func.span),
            name_span: self.ws_span(func.span),
            disabled: false,
            event: wir::Event::Subroutine(subroutine),
            conditions: Vec::new(),
            actions,
        });
    }

    fn lower_event(&mut self, id: HirExprId) -> Option<wir::Event> {
        let expr = self.hir.expr(id)?.clone();
        let HirExprKind::External {
            binding: Some(ExternalBinding::Event(info)),
            ..
        } = expr.kind
        else {
            self.unsupported(
                expr.span,
                "rule event is not a canonical Workshop event binding",
            );
            return None;
        };
        match info.canonical_id.as_str() {
            "global" => Some(wir::Event::Global),
            "eachPlayer" => Some(wir::Event::EachPlayer),
            "playerDealtDamage" => Some(self.player_event(wir::PlayerEventKind::DealtDamage)),
            "playerDealtFinalBlow" => Some(self.player_event(wir::PlayerEventKind::DealtFinalBlow)),
            "playerDealtHealing" => Some(self.player_event(wir::PlayerEventKind::DealtHealing)),
            "playerDied" => Some(self.player_event(wir::PlayerEventKind::Died)),
            "playerEarnedElimination" => {
                Some(self.player_event(wir::PlayerEventKind::EarnedElimination))
            }
            "playerJoined" => Some(self.player_event(wir::PlayerEventKind::Joined)),
            "playerLeft" => Some(self.player_event(wir::PlayerEventKind::Left)),
            "playerReceivedHealing" => {
                Some(self.player_event(wir::PlayerEventKind::ReceivedHealing))
            }
            "playerTookDamage" => Some(self.player_event(wir::PlayerEventKind::TookDamage)),
            "subroutine" => {
                self.unsupported(
                    expr.span,
                    "subroutine event requires a canonical subroutine reference",
                );
                None
            }
            other => {
                self.unsupported(
                    expr.span,
                    format!("unsupported canonical Workshop event '{other}'"),
                );
                None
            }
        }
    }

    fn player_event(&self, kind: wir::PlayerEventKind) -> wir::Event {
        wir::Event::Player {
            kind,
            team: wir::EventTeam::All,
            target: wir::EventTarget::All,
        }
    }

    fn lower_actions(&mut self, block: &crate::hir::HirBlock) -> Vec<wir::ActionId> {
        let mut actions = Vec::new();
        for stmt in &block.stmts {
            actions.extend(self.lower_stmt(stmt));
        }
        actions
    }

    fn lower_stmt(&mut self, stmt: &HirStmt) -> Vec<wir::ActionId> {
        match &stmt.kind {
            HirStmtKind::Block(block) => self.lower_actions(block),
            HirStmtKind::Expr(expr) => self.lower_expr_action(*expr),
            HirStmtKind::Assign { target, op, value } => {
                self.lower_assignment(*target, *op, *value, stmt.span)
            }
            HirStmtKind::If { cond, then, els } => {
                let Ok(condition) = self.lower_value(*cond) else {
                    return Vec::new();
                };
                let then_body = self.lower_stmt(then);
                let else_body = els.as_ref().map(|body| self.lower_stmt(body));
                vec![self.out.actions.push(wir::Action::If {
                    branches: vec![wir::IfBranch {
                        condition,
                        body: then_body,
                    }],
                    else_body,
                    span: self.ws_span(stmt.span),
                })]
            }
            HirStmtKind::While { cond, body } => {
                let Ok(condition) = self.lower_value(*cond) else {
                    return Vec::new();
                };
                let body = self.lower_stmt(body);
                vec![self.out.actions.push(wir::Action::While {
                    condition,
                    body,
                    span: self.ws_span(stmt.span),
                })]
            }
            HirStmtKind::AutoFor {
                var,
                start,
                end,
                step,
                body,
            } => {
                let Ok(start) = self.lower_value(*start) else {
                    return Vec::new();
                };
                let Ok(stop) = self.lower_value(*end) else {
                    return Vec::new();
                };
                let Ok(step) = self.lower_value(*step) else {
                    return Vec::new();
                };
                let body = self.lower_stmt(body);
                match (
                    self.global_vars.get(var).copied(),
                    self.player_vars.get(var).copied(),
                ) {
                    (Some(variable), _) => {
                        vec![self.out.actions.push(wir::Action::ForGlobalVariable {
                            variable,
                            start,
                            stop,
                            step,
                            body,
                            span: self.ws_span(stmt.span),
                            target_span: self
                                .hir
                                .vars
                                .get(*var as usize)
                                .and_then(|v| self.ws_span(v.span)),
                        })]
                    }
                    (None, Some(variable)) => {
                        let player = self.out.values.push(wir::ValueNode::new(
                            wir::Value::EventPlayer,
                            self.ws_span(stmt.span),
                        ));
                        vec![self.out.actions.push(wir::Action::ForPlayerVariable {
                            player,
                            variable,
                            start,
                            stop,
                            step,
                            body,
                            span: self.ws_span(stmt.span),
                        })]
                    }
                    _ => {
                        self.unsupported(
                            stmt.span,
                            "for-loop variable has no canonical Workshop storage",
                        );
                        Vec::new()
                    }
                }
            }
            HirStmtKind::VarDecl { .. } => {
                self.unsupported(
                    stmt.span,
                    "rule-local variable declarations have no canonical WIR storage",
                );
                Vec::new()
            }
            HirStmtKind::For { .. }
            | HirStmtKind::Foreach { .. }
            | HirStmtKind::Switch { .. }
            | HirStmtKind::Return { .. }
            | HirStmtKind::Break
            | HirStmtKind::Continue
            | HirStmtKind::Delete { .. }
            | HirStmtKind::Hook { .. }
            | HirStmtKind::Error => {
                self.unsupported(
                    stmt.span,
                    "statement is not supported by the core Workshop lowering",
                );
                Vec::new()
            }
        }
    }

    fn lower_expr_action(&mut self, id: HirExprId) -> Vec<wir::ActionId> {
        let Some(expr) = self.hir.expr(id).cloned() else {
            self.unsupported(self.fallback_span(), format!("unknown HIR expression {id}"));
            return Vec::new();
        };
        match expr.kind {
            HirExprKind::Assign { target, op, value } => {
                self.lower_assignment(target, op, value, expr.span)
            }
            HirExprKind::Call { target, args } => match target {
                CallTarget::External {
                    binding: Some(ExternalBinding::Action(info)),
                    ..
                } => {
                    let Ok(args) = self.lower_args(&args) else {
                        return Vec::new();
                    };
                    vec![self.out.actions.push(wir::Action::Call {
                        name: info.canonical_id,
                        args,
                        span: self.ws_span(expr.span),
                    })]
                }
                CallTarget::Func(fid) => self.call_subroutine(fid, expr.span),
                _ => {
                    self.unsupported(
                        expr.span,
                        "expression statement is not a canonical Workshop action",
                    );
                    Vec::new()
                }
            },
            _ => {
                self.unsupported(
                    expr.span,
                    "expression statement is not a canonical Workshop action",
                );
                Vec::new()
            }
        }
    }

    fn call_subroutine(&mut self, fid: HirFuncId, span: Span) -> Vec<wir::ActionId> {
        if let Some(subroutine) = self.subroutines.get(&fid).copied() {
            vec![self.out.actions.push(wir::Action::CallSubroutine {
                subroutine,
                span: self.ws_span(span),
                callee_span: self.ws_span(span),
            })]
        } else {
            self.unsupported(span, "call target is not a canonical Workshop subroutine");
            Vec::new()
        }
    }

    fn lower_assignment(
        &mut self,
        target: HirExprId,
        op: AssignOp,
        value: HirExprId,
        span: Span,
    ) -> Vec<wir::ActionId> {
        let Some(target_expr) = self.hir.expr(target).cloned() else {
            self.unsupported(span, "assignment target is not a known HIR expression");
            return Vec::new();
        };
        let HirExprKind::VarRef { var } = target_expr.kind else {
            self.unsupported(span, "assignment target is not a Workshop variable");
            return Vec::new();
        };
        let Ok(value) = self.lower_value(value) else {
            return Vec::new();
        };
        let target_span = self
            .hir
            .vars
            .get(var as usize)
            .and_then(|v| self.ws_span(v.span));
        let modify = self.modify_op(op, span);
        match (
            self.global_vars.get(&var).copied(),
            self.player_vars.get(&var).copied(),
        ) {
            (Some(variable), _) => {
                if let Some(op) = modify {
                    vec![self.out.actions.push(wir::Action::ModifyGlobalVariable {
                        variable,
                        op,
                        value,
                        span: self.ws_span(span),
                        target_span,
                    })]
                } else {
                    vec![self.out.actions.push(wir::Action::SetGlobalVariable {
                        variable,
                        value,
                        span: self.ws_span(span),
                        target_span,
                    })]
                }
            }
            (None, Some(variable)) => {
                let player = self.out.values.push(wir::ValueNode::new(
                    wir::Value::EventPlayer,
                    self.ws_span(span),
                ));
                if let Some(op) = modify {
                    vec![self.out.actions.push(wir::Action::ModifyPlayerVariable {
                        player,
                        variable,
                        op,
                        value,
                        span: self.ws_span(span),
                        target_span,
                    })]
                } else {
                    vec![self.out.actions.push(wir::Action::SetPlayerVariable {
                        player,
                        variable,
                        value,
                        span: self.ws_span(span),
                        target_span,
                    })]
                }
            }
            _ => {
                self.unsupported(span, "assignment target has no canonical Workshop storage");
                Vec::new()
            }
        }
    }

    fn modify_op(&mut self, op: AssignOp, _span: Span) -> Option<wir::ModifyOp> {
        match op {
            AssignOp::Assign => None,
            AssignOp::Add => Some(wir::ModifyOp::Add),
            AssignOp::Sub => Some(wir::ModifyOp::Subtract),
            AssignOp::Mul => Some(wir::ModifyOp::Multiply),
            AssignOp::Div => Some(wir::ModifyOp::Divide),
            AssignOp::Mod => Some(wir::ModifyOp::Modulo),
            AssignOp::Pow => Some(wir::ModifyOp::RaiseToPower),
        }
    }

    fn lower_args(&mut self, args: &[HirArg]) -> Result<Vec<wir::ValueId>, ()> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                HirArg::Pos(value) => values.push(self.lower_value(*value)?),
                HirArg::Named { .. } => {
                    let span = self.hir_arg_span(arg);
                    self.unsupported(
                        span,
                        "named Workshop action arguments require canonical parameter ordering",
                    );
                    return Err(());
                }
            }
        }
        Ok(values)
    }

    fn lower_value(&mut self, id: HirExprId) -> Result<wir::ValueId, ()> {
        let expr = self.hir.expr(id).cloned().ok_or_else(|| {
            self.unsupported(self.fallback_span(), format!("unknown HIR expression {id}"));
        })?;
        let span = self.ws_span(expr.span);
        let value = match expr.kind {
            HirExprKind::Literal(literal) => match literal {
                LiteralValue::Number(value) => wir::Value::Number {
                    value,
                    text: format_number(value),
                },
                LiteralValue::Str(value) => wir::Value::String(unquote(&value)),
                LiteralValue::Bool(value) => wir::Value::Bool(value),
                LiteralValue::Null => wir::Value::Null,
            },
            HirExprKind::VarRef { var } => {
                if let Some(variable) = self.global_vars.get(&var).copied() {
                    wir::Value::GlobalVariable(variable)
                } else if let Some(variable) = self.player_vars.get(&var).copied() {
                    let player = self
                        .out
                        .values
                        .push(wir::ValueNode::new(wir::Value::EventPlayer, span));
                    wir::Value::PlayerVariable { player, variable }
                } else {
                    self.unsupported(
                        expr.span,
                        "value references a variable without canonical Workshop storage",
                    );
                    return Err(());
                }
            }
            HirExprKind::External {
                binding: Some(binding),
                ..
            } => self.value_from_binding(binding, Vec::new(), expr.span)?,
            HirExprKind::Call { target, args } => {
                let args = self.lower_args(&args)?;
                match target {
                    CallTarget::External {
                        binding: Some(binding),
                        ..
                    } => self.value_from_binding(binding, args, expr.span)?,
                    CallTarget::BuiltinArrayMethod { member, base } => {
                        let mut all = vec![self.lower_value(base)?];
                        all.extend(args);
                        let name = match member {
                            crate::hir::BuiltinArrayMember::Length => "countOf",
                            crate::hir::BuiltinArrayMember::IndexOf => "indexOfArrayValue",
                            crate::hir::BuiltinArrayMember::First => "firstOf",
                            crate::hir::BuiltinArrayMember::Last => "lastOf",
                            crate::hir::BuiltinArrayMember::Random => "randomValueInArray",
                            crate::hir::BuiltinArrayMember::Contains => "arrayContains",
                            crate::hir::BuiltinArrayMember::SortedArray => "sortedArray",
                            crate::hir::BuiltinArrayMember::FilteredArray => "filteredArray",
                            _ => {
                                self.unsupported(
                                    expr.span,
                                    "array method has no canonical core WIR lowering",
                                );
                                return Err(());
                            }
                        };
                        wir::Value::Call {
                            name: name.to_string(),
                            args: all,
                        }
                    }
                    _ => {
                        self.unsupported(
                            expr.span,
                            "value call has no canonical Workshop value binding",
                        );
                        return Err(());
                    }
                }
            }
            HirExprKind::Binary { op, lhs, rhs } => {
                let name = binary_name(op);
                let args = vec![self.lower_value(lhs)?, self.lower_value(rhs)?];
                wir::Value::Call {
                    name: name.to_string(),
                    args,
                }
            }
            HirExprKind::Unary { op, operand } => match op {
                UnaryOp::Negate => {
                    let operand = self.lower_value(operand)?;
                    let minus_one = self.out.values.push(wir::ValueNode::new(
                        wir::Value::Number {
                            value: -1.0,
                            text: "-1".to_string(),
                        },
                        span,
                    ));
                    wir::Value::Call {
                        name: "multiply".to_string(),
                        args: vec![minus_one, operand],
                    }
                }
                UnaryOp::Not => wir::Value::Call {
                    name: "not".to_string(),
                    args: vec![self.lower_value(operand)?],
                },
                UnaryOp::Indirect => {
                    self.unsupported(expr.span, "Workshop indirection has no core WIR lowering");
                    return Err(());
                }
            },
            HirExprKind::ArrayLit { elems } => {
                let mut values = Vec::with_capacity(elems.len());
                for elem in elems {
                    values.push(self.lower_value(elem)?);
                }
                wir::Value::Array(values)
            }
            HirExprKind::Index { base, index } => wir::Value::Call {
                name: "valueInArray".to_string(),
                args: vec![self.lower_value(base)?, self.lower_value(index)?],
            },
            HirExprKind::Ternary { cond, then, els } => wir::Value::Call {
                name: "ifThenElse".to_string(),
                args: vec![
                    self.lower_value(cond)?,
                    self.lower_value(then)?,
                    self.lower_value(els)?,
                ],
            },
            HirExprKind::Convert { from, .. } | HirExprKind::Cast { expr: from, .. } => {
                return self.lower_value(from);
            }
            HirExprKind::StrInterp { .. }
            | HirExprKind::Assign { .. }
            | HirExprKind::Member { .. }
            | HirExprKind::FunctionValue { .. }
            | HirExprKind::New { .. }
            | HirExprKind::StructLit { .. }
            | HirExprKind::EnumCtor { .. }
            | HirExprKind::Async { .. }
            | HirExprKind::This { .. }
            | HirExprKind::Postfix { .. }
            | HirExprKind::Error
            | HirExprKind::External { binding: None, .. } => {
                self.unsupported(
                    expr.span,
                    "expression has no core canonical Workshop value lowering",
                );
                return Err(());
            }
        };
        Ok(self.out.values.push(wir::ValueNode::new(value, span)))
    }

    fn value_from_binding(
        &mut self,
        binding: ExternalBinding,
        args: Vec<wir::ValueId>,
        span: Span,
    ) -> Result<wir::Value, ()> {
        match binding {
            ExternalBinding::Value(info) => {
                if let Some((value_type, value)) = info.canonical_id.split_once('.') {
                    if args.is_empty() {
                        return Ok(wir::Value::Enum {
                            value_type: value_type.to_string(),
                            value: value.to_string(),
                        });
                    }
                }
                Ok(wir::Value::Call {
                    name: info.canonical_id,
                    args,
                })
            }
            ExternalBinding::Type(info) if info.constant => {
                self.unsupported(span, "enum domain reference is not a canonical enum member");
                Err(())
            }
            other => {
                self.unsupported(
                    span,
                    format!(
                        "binding {:?} cannot be lowered as a Workshop value",
                        binding_kind(&other)
                    ),
                );
                Err(())
            }
        }
    }

    fn validate_output(&mut self) {
        if let Err(error) = self.out.validate() {
            self.unsupported(
                self.fallback_span(),
                format!("canonical WIR validation failed: {error}"),
            );
        }
        if let Ok(catalog) = Catalog::builtin() {
            if let Err(error) = workshop_rs::validate::validate_canonical_ids(&self.out, &catalog) {
                self.unsupported(
                    self.fallback_span(),
                    format!("canonical Workshop identity validation failed: {error}"),
                );
            }
        }
    }

    fn unsupported(&mut self, span: Span, message: impl Into<String>) {
        self.diagnostics
            .push(error(Phase::Hir, "HI018", span, message));
    }

    fn has_new_errors(&self, start: usize) -> bool {
        self.diagnostics
            .get(start..)
            .unwrap_or_default()
            .iter()
            .any(Diagnostic::is_error)
    }

    fn fallback_span(&self) -> Span {
        Span::new(FileId(0), 0, 0)
    }

    fn ws_span(&self, span: Span) -> Option<WorkshopSpan> {
        let source = self.sources.files().nth(span.file.0 as usize)?;
        let start = self.sources.line_col(span, span.start);
        let end = self.sources.line_col(span, span.end);
        let file = workshop_rs::ids::Id::from_index(span.file.0 as usize);
        let _ = source;
        Some(WorkshopSpan::new(
            file,
            Position::new(start.line, start.col),
            Position::new(end.line, end.col),
        ))
    }

    fn hir_arg_span(&self, arg: &HirArg) -> Span {
        let id = match arg {
            HirArg::Pos(id) | HirArg::Named { value: id, .. } => *id,
        };
        self.hir
            .expr(id)
            .map(|e| e.span)
            .unwrap_or_else(|| self.fallback_span())
    }
}

fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Sub => "subtract",
        BinaryOp::Mul => "multiply",
        BinaryOp::Div => "divide",
        BinaryOp::Mod => "modulo",
        BinaryOp::Pow => "raiseToPower",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
    }
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn unquote(value: &str) -> String {
    if value.len() >= 2 && (value.starts_with('"') || value.starts_with('\'')) {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn binding_kind(binding: &ExternalBinding) -> &'static str {
    match binding {
        ExternalBinding::Value(_) => "value",
        ExternalBinding::Action(_) => "action",
        ExternalBinding::Event(_) => "event",
        ExternalBinding::Type(_) => "type",
        ExternalBinding::Namespace => "namespace",
    }
}
