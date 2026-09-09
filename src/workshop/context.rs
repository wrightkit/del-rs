use crate::semantic::provider::ExternalBinding;
use crate::semantic::resolve::Resolution;
use crate::semantic::SemanticProgram;
use crate::span::Span;
use crate::syntax::ast::{self, Expr, ExprKind, FuncBody, Item, ItemKind, Stmt, StmtKind};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ExternalKey {
    span: Span,
    name: String,
    namespace: Vec<String>,
}

/// DEL-owned handoff from semantic provider resolution to canonical WIR
/// lowering. It contains only the resolved external bindings and their source
/// lookup keys required by the Workshop backend.
pub(super) struct WorkshopLoweringContext {
    bindings: HashMap<ExternalKey, ExternalBinding>,
}

impl WorkshopLoweringContext {
    pub(super) fn from_semantic(semantic: &SemanticProgram) -> Self {
        let mut context = Self {
            bindings: HashMap::new(),
        };
        for ast in semantic.asts.values() {
            for item in &ast.items {
                context.collect_item(item, semantic);
            }
        }
        context
    }

    pub(super) fn lookup(
        &self,
        span: Span,
        name: &str,
        namespace: &[String],
    ) -> Option<ExternalBinding> {
        self.bindings
            .get(&ExternalKey {
                span,
                name: name.to_string(),
                namespace: namespace.to_vec(),
            })
            .cloned()
    }

    fn collect_item(&mut self, item: &Item, semantic: &SemanticProgram) {
        match &item.kind {
            ItemKind::Rule(rule) => {
                self.collect_expr(&rule.name, semantic);
                if let Some(sort_order) = &rule.sort_order {
                    self.collect_expr(sort_order, semantic);
                }
                for setting in &rule.settings {
                    self.collect_expr(setting, semantic);
                }
                if let Some(event) = &rule.event {
                    self.collect_expr(event, semantic);
                }
                for condition in &rule.conditions {
                    self.collect_expr(&condition.expr, semantic);
                }
                self.collect_stmt(&rule.body, semantic);
            }
            ItemKind::VanillaRule(rule) => {
                if let Some(name) = &rule.name {
                    self.collect_expr(name, semantic);
                }
            }
            ItemKind::Var(var) => self.collect_var(var, semantic),
            ItemKind::Function(function) => self.collect_function(function, semantic),
            ItemKind::TypeDecl(decl) => {
                for member in &decl.members {
                    match &member.kind {
                        ast::MemberDeclKind::Field(var) => self.collect_var(var, semantic),
                        ast::MemberDeclKind::Method(function) => {
                            self.collect_function(function, semantic)
                        }
                        ast::MemberDeclKind::Constructor(constructor) => {
                            if let Some(subroutine) = &constructor.subroutine {
                                self.collect_expr(subroutine, semantic);
                            }
                            self.collect_block(&constructor.body, semantic);
                        }
                        ast::MemberDeclKind::EnumMember(member) => {
                            if let Some(discriminant) = &member.discriminant {
                                self.collect_expr(discriminant, semantic);
                            }
                        }
                    }
                }
            }
            ItemKind::Import(import) => self.collect_expr(&import.path, semantic),
            ItemKind::VarReservation(reservation) => {
                for name in &reservation.names {
                    self.collect_expr(name, semantic);
                }
            }
            ItemKind::Hook { target, value } => {
                self.collect_expr(target, semantic);
                self.collect_expr(value, semantic);
            }
            ItemKind::VanillaBlock(_) | ItemKind::TypeAlias(_) | ItemKind::Error { .. } => {}
        }
    }

    fn collect_function(&mut self, function: &ast::FunctionDecl, semantic: &SemanticProgram) {
        if let Some(subroutine) = &function.attrs.subroutine {
            self.collect_expr(&subroutine.rule_name, semantic);
        }
        for param in &function.params {
            if let Some(default) = &param.default {
                self.collect_expr(default, semantic);
            }
        }
        match &function.body {
            FuncBody::Block(block) => self.collect_block(block, semantic),
            FuncBody::Expr(expr) => self.collect_expr(expr, semantic),
            FuncBody::None => {}
        }
    }

    fn collect_var(&mut self, var: &ast::VarDecl, semantic: &SemanticProgram) {
        if let Some(var_id) = &var.var_id {
            self.collect_expr(var_id, semantic);
        }
        if let Some((_, init)) = &var.init {
            self.collect_expr(init, semantic);
        }
    }

    fn collect_block(&mut self, block: &ast::BlockStmt, semantic: &SemanticProgram) {
        for stmt in &block.stmts {
            self.collect_stmt(stmt, semantic);
        }
    }

    fn collect_stmt(&mut self, stmt: &Stmt, semantic: &SemanticProgram) {
        match &stmt.kind {
            StmtKind::Block(block) => self.collect_block(block, semantic),
            StmtKind::Var(var) => self.collect_var(var, semantic),
            StmtKind::If { cond, then, els } => {
                self.collect_expr(cond, semantic);
                self.collect_stmt(then, semantic);
                if let Some(els) = els {
                    self.collect_stmt(els, semantic);
                }
            }
            StmtKind::While { cond, body } => {
                self.collect_expr(cond, semantic);
                self.collect_stmt(body, semantic);
            }
            StmtKind::For(for_stmt) => {
                if let Some(init) = &for_stmt.init {
                    self.collect_stmt(init, semantic);
                }
                if let Some(cond) = &for_stmt.cond {
                    self.collect_expr(cond, semantic);
                }
                if let Some(step) = &for_stmt.step {
                    self.collect_stmt(step, semantic);
                }
                self.collect_stmt(&for_stmt.body, semantic);
            }
            StmtKind::Foreach {
                collection, body, ..
            } => {
                self.collect_expr(collection, semantic);
                self.collect_stmt(body, semantic);
            }
            StmtKind::Switch(switch) => {
                self.collect_expr(&switch.scrutinee, semantic);
                for arm in &switch.arms {
                    if let Some(label) = &arm.label {
                        self.collect_expr(label, semantic);
                    }
                    for stmt in &arm.stmts {
                        self.collect_stmt(stmt, semantic);
                    }
                }
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    self.collect_expr(value, semantic);
                }
            }
            StmtKind::Expr(expr) => self.collect_expr(expr, semantic),
            StmtKind::Delete { target } => self.collect_expr(target, semantic),
            StmtKind::Hook { target, value } => {
                self.collect_expr(target, semantic);
                self.collect_expr(value, semantic);
            }
            StmtKind::Break | StmtKind::Continue | StmtKind::Error { .. } => {}
        }
    }

    fn collect_expr(&mut self, expr: &Expr, semantic: &SemanticProgram) {
        if let Some(Resolution::External(binding)) = semantic.resolution.get(&expr.id) {
            if let Some((name, namespace)) = external_name(expr) {
                self.bindings.insert(
                    ExternalKey {
                        span: expr.span,
                        name,
                        namespace,
                    },
                    binding.clone(),
                );
            }
        }
        match &expr.kind {
            ExprKind::Member { base, .. } => self.collect_expr(base, semantic),
            ExprKind::Index { base, index } => {
                self.collect_expr(base, semantic);
                self.collect_expr(index, semantic);
            }
            ExprKind::Call(call) => {
                if let Some(Resolution::External(binding)) = semantic.resolution.get(&expr.id) {
                    if let Some((name, namespace)) = external_name(&call.callee) {
                        for span in [expr.span, call.callee.span] {
                            self.bindings.insert(
                                ExternalKey {
                                    span,
                                    name: name.clone(),
                                    namespace: namespace.clone(),
                                },
                                binding.clone(),
                            );
                        }
                    }
                }
                self.collect_expr(&call.callee, semantic);
                for arg in &call.args {
                    self.collect_expr(&arg.value, semantic);
                }
            }
            ExprKind::Unary { operand, .. }
            | ExprKind::Cast { expr: operand, .. }
            | ExprKind::Async { call: operand, .. }
            | ExprKind::Postfix { operand, .. } => self.collect_expr(operand, semantic),
            ExprKind::Binary { lhs, rhs, .. }
            | ExprKind::Assign {
                target: lhs,
                value: rhs,
                ..
            } => {
                self.collect_expr(lhs, semantic);
                self.collect_expr(rhs, semantic);
            }
            ExprKind::Ternary { cond, then, els } => {
                self.collect_expr(cond, semantic);
                self.collect_expr(then, semantic);
                self.collect_expr(els, semantic);
            }
            ExprKind::New { args, .. } => {
                for arg in args {
                    self.collect_expr(&arg.value, semantic);
                }
            }
            ExprKind::ArrayLit { elems } => {
                for elem in elems {
                    self.collect_expr(elem, semantic);
                }
            }
            ExprKind::StructLit(struct_lit) => {
                for field in &struct_lit.fields {
                    self.collect_expr(&field.value, semantic);
                }
                if let Some(base) = &struct_lit.base {
                    self.collect_expr(base, semantic);
                }
                if let Some(value) = &struct_lit.single_value {
                    self.collect_expr(value, semantic);
                }
            }
            ExprKind::Lambda(lambda) => match &lambda.body {
                ast::LambdaBody::Expr(expr) => self.collect_expr(expr, semantic),
                ast::LambdaBody::Block(block) => self.collect_block(block, semantic),
            },
            ExprKind::StrInterp { args, .. } | ExprKind::Interp { args, .. } => {
                for arg in args {
                    self.collect_expr(arg, semantic);
                }
            }
            ExprKind::Is { operand, .. } => self.collect_expr(operand, semantic),
            ExprKind::JsonImport { path, .. } => self.collect_expr(path, semantic),
            ExprKind::VanillaTarget { index, .. } => {
                if let Some(index) = index {
                    self.collect_expr(index, semantic);
                }
            }
            ExprKind::Number(_)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::Null
            | ExprKind::Ident(_)
            | ExprKind::This
            | ExprKind::Root
            | ExprKind::Error { .. } => {}
        }
    }
}

fn external_name(expr: &Expr) -> Option<(String, Vec<String>)> {
    match &expr.kind {
        ExprKind::Ident(ident) => Some((ident.name.clone(), Vec::new())),
        ExprKind::Member { base, name } => Some((name.name.clone(), member_namespace(base))),
        _ => None,
    }
}

fn member_namespace(base: &Expr) -> Vec<String> {
    match &base.kind {
        ExprKind::Ident(ident) => vec![ident.name.clone()],
        ExprKind::Member { base, name } => {
            let mut namespace = member_namespace(base);
            namespace.push(name.name.clone());
            namespace
        }
        _ => Vec::new(),
    }
}
