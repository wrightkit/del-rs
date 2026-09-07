//! Rule, enum, and body-level semantic policies.

use super::*;
use crate::semantic::types::is_constant_or_parallel;
use crate::syntax::ast::*;

impl<'a> Checker<'a> {
    pub(super) fn collect_enum_keys(&mut self) -> Vec<(Expr, bool)> {
        let mut out = Vec::new();
        for file in &self.program.project.files {
            let Some(parsed) = self.program.asts.get(file) else {
                continue;
            };
            for item in &parsed.items {
                if let ItemKind::TypeDecl(t) = &item.kind {
                    if t.kind != TypeDeclKind::Enum {
                        continue;
                    }
                    for m in &t.members {
                        if let MemberDeclKind::EnumMember(e) = &m.kind {
                            if let Some(d) = &e.discriminant {
                                out.push((d.clone(), !t.single));
                            }
                        }
                    }
                }
            }
        }
        out
    }

    pub(super) fn check_enum_member_keys(&mut self) {
        // Enum member keys must not be constant or parallel (SM042).
        let enum_keys = self.collect_enum_keys();
        for (discriminant, parallel) in enum_keys {
            let ty = self.check_expr(&discriminant);
            if parallel
                && (is_constant_or_parallel(&ty)
                    || (ty.is_external()
                        && matches!(
                            discriminant.kind,
                            ExprKind::Member { .. } | ExprKind::Ident(_)
                        )))
            {
                self.err(
                    "SM042",
                    discriminant.span,
                    "the key of an enum member cannot be a constant or parallel data type",
                );
            }
        }
    }

    pub(super) fn find_var_init(&mut self, nid: NodeId) -> Option<(InitKind, Expr)> {
        for file in &self.program.project.files {
            let Some(parsed) = self.program.asts.get(file) else {
                continue;
            };
            for item in &parsed.items {
                if let ItemKind::Var(v) = &item.kind {
                    if v.name.id == nid {
                        return v.init.clone();
                    }
                }
                if let ItemKind::TypeDecl(t) = &item.kind {
                    for m in &t.members {
                        if let MemberDeclKind::Field(v) = &m.kind {
                            if v.name.id == nid {
                                return v.init.clone();
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub(super) fn check_rule_body(&mut self, body_node: NodeId) {
        let decls: Vec<(NodeId, RuleDecl)> = self.collect_rules();
        if let Some((_, rule)) = decls.into_iter().find(|(n, _)| *n == body_node) {
            for cond in &rule.conditions {
                let ty = self.check_expr(&cond.expr);
                if !self.is_boolish(&ty) {
                    self.err(
                        "SM019",
                        cond.expr.span,
                        format!(
                            "rule condition must be bool-compatible, found {}",
                            ty.describe()
                        ),
                    );
                }
                if is_constant_or_parallel(&ty)
                    || (ty.is_external()
                        && matches!(cond.expr.kind, ExprKind::Member { .. } | ExprKind::Ident(_)))
                {
                    self.err(
                        "SM046",
                        cond.expr.span,
                        "the value of a rule condition cannot be a constant or parallel value",
                    );
                }
            }
            if let Some(ev) = &rule.event {
                let _ = self.check_expr(ev);
            }
            for s in &rule.settings {
                let _ = self.check_expr(s);
            }
            if let Some(so) = &rule.sort_order {
                let _ = self.check_expr(so);
            }
            let saved = self.ref_context;
            self.ref_context = true;
            self.check_statement(&rule.body);
            self.ref_context = saved;
        }
    }

    pub(super) fn collect_rules(&self) -> Vec<(NodeId, RuleDecl)> {
        let mut out = Vec::new();
        for file in &self.program.project.files {
            if let Some(ast) = self.program.asts.get(file) {
                for item in &ast.items {
                    if let ItemKind::Rule(r) = &item.kind {
                        out.push((item.id, r.clone()));
                    }
                }
            }
        }
        out
    }

    pub(super) fn check_function_body(&mut self, body_node: NodeId) {
        let func_sym = self.program.function_of_body.get(&body_node).copied();
        let class = self
            .program
            .class_of_body
            .get(&body_node)
            .copied()
            .filter(|c| *c != u32::MAX);
        self.cur_function = func_sym;
        self.cur_class = class;
        self.ref_context = self
            .program
            .ref_of_body
            .get(&body_node)
            .copied()
            .unwrap_or(false);
        self.ret_ty = self.program.ret_of_body.get(&body_node).cloned();

        let stmts: Vec<Stmt> = self.collect_body_stmts(body_node);
        for s in &stmts {
            self.check_statement(s);
        }
        self.cur_function = None;
        self.cur_class = None;
        self.ref_context = false;
        self.ret_ty = None;
    }

    pub(super) fn collect_body_stmts(&self, body_node: NodeId) -> Vec<Stmt> {
        for file in &self.program.project.files {
            let Some(parsed) = self.program.asts.get(file) else {
                continue;
            };
            for item in &parsed.items {
                match &item.kind {
                    ItemKind::Function(f) if f.name.id == body_node => {
                        return body_stmts(f.body.clone());
                    }
                    ItemKind::TypeDecl(t) => {
                        for m in &t.members {
                            match &m.kind {
                                MemberDeclKind::Method(f) if f.name.id == body_node => {
                                    return body_stmts(f.body.clone());
                                }
                                MemberDeclKind::Constructor(c) if m.id == body_node => {
                                    return c.body.stmts.clone();
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Vec::new()
    }
}

fn body_stmts(body: FuncBody) -> Vec<Stmt> {
    match body {
        FuncBody::Block(b) => b.stmts,
        FuncBody::Expr(e) => vec![Stmt {
            id: e.id,
            span: e.span,
            kind: StmtKind::Return { value: Some(e) },
        }],
        FuncBody::None => Vec::new(),
    }
}
