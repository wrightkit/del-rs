use super::*;
use crate::semantic::symbols::*;
use crate::semantic::types::*;
use crate::syntax::ast::*;

impl Checker<'_> {
    pub fn check_statement(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Block(b) => {
                let scope = self
                    .program
                    .tables
                    .push_scope(self.scope(), ScopeKind::Block);
                self.scopes.push(scope);
                for s in &b.stmts {
                    self.check_statement(s);
                }
                self.scopes.pop();
            }
            StmtKind::Var(v) => {
                let ty = self.decl_type(v);
                let already = self
                    .program
                    .tables
                    .scope(self.scope())
                    .entries
                    .contains_key(&v.name.name);
                if !already {
                    let sym = Symbol {
                        name: v.name.name.clone(),
                        kind: SymbolKind::Variable,
                        span: v.name.span,
                        decl: v.name.id,
                        visibility: Visibility::Public,
                        ty: ty.clone(),
                        owner: None,
                        flags: SymbolFlags {
                            const_init: v.is_const_init,
                            extended: v.extended,
                            ..Default::default()
                        },
                    };
                    if self.program.tables.declare(self.scope(), sym).is_err() {
                        self.err(
                            "SM001",
                            v.name.span,
                            format!("duplicate declaration of '{}'", v.name.name),
                        );
                    }
                }
                if let Some((_, init)) = &v.init {
                    let init_ty = if ty.is_external() {
                        self.check_expr(init)
                    } else {
                        self.check_expr_with_hint(init, ty.clone())
                    };
                    // `define` infers the variable's type from its initializer
                    // (null initializers stay Any: `define x = null; x = v;`).
                    if matches!(v.kind, VarDeclKind::Define) {
                        if let Some(&sid) = self
                            .program
                            .tables
                            .scope(self.scope())
                            .entries
                            .get(&v.name.name)
                            .and_then(|ids| ids.first())
                        {
                            let inferred = if init_ty == Type::Null {
                                Type::Any
                            } else {
                                init_ty.clone()
                            };
                            self.program.tables.symbols[sid as usize].ty = inferred;
                        }
                    }
                    if !(self.is_assignable(&init_ty, &ty)
                        || ty.is_external()
                        || ty.is_error()
                        || init_ty.is_external()
                        || ty == Type::Any && matches!(v.kind, VarDeclKind::Define))
                    {
                        self.err(
                            "SM051",
                            init.span,
                            format!(
                                "cannot initialize '{}' of type {} with a value of type {}",
                                v.name.name,
                                ty.describe(),
                                init_ty.describe()
                            ),
                        );
                    }
                }
            }
            StmtKind::If { cond, then, els } => {
                let ty = self.check_expr(cond);
                if !self.is_boolish(&ty) {
                    self.err(
                        "SM052",
                        cond.span,
                        format!(
                            "if condition must be bool-compatible, found {}",
                            ty.describe()
                        ),
                    );
                }
                self.check_statement(then);
                if let Some(e) = els {
                    self.check_statement(e);
                }
            }
            StmtKind::While { cond, body } => {
                let ty = self.check_expr(cond);
                if !self.is_boolish(&ty) {
                    self.err(
                        "SM052",
                        cond.span,
                        format!(
                            "while condition must be bool-compatible, found {}",
                            ty.describe()
                        ),
                    );
                }
                self.loop_depth += 1;
                self.check_statement(body);
                self.loop_depth -= 1;
            }
            StmtKind::For(f) => {
                // Auto-for (upstream Loops.cs): the step is an expression
                // statement; the "condition" is the end value and the step is
                // the increment — no bool/statement semantics apply.
                let is_auto_for = matches!(
                    f.step.as_deref().map(|s| &s.kind),
                    Some(StmtKind::Expr(e)) if !matches!(e.kind, ExprKind::Assign { .. })
                );
                self.loop_depth += 1;
                if let Some(init) = &f.init {
                    self.check_statement(init);
                }
                if let Some(c) = &f.cond {
                    let ty = self.check_expr(c);
                    if !is_auto_for && !self.is_boolish(&ty) {
                        self.err(
                            "SM052",
                            c.span,
                            format!(
                                "for condition must be bool-compatible, found {}",
                                ty.describe()
                            ),
                        );
                    }
                }
                if let Some(step) = &f.step {
                    if is_auto_for {
                        if let StmtKind::Expr(e) = &step.kind {
                            let _ = self.check_expr(e);
                        }
                    } else {
                        self.check_statement(step);
                    }
                }
                self.check_statement(&f.body);
                self.loop_depth -= 1;
            }
            StmtKind::Foreach {
                var,
                collection,
                body,
            } => {
                let coll_ty = self.check_expr(collection);
                let elem = coll_ty
                    .array_element()
                    .cloned()
                    .or_else(|| {
                        // Vectors iterate their components (corpus
                        // PathfindEditor `foreach (Vector p in v)`).
                        if coll_ty == Type::Vector {
                            Some(Type::Number)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        if !coll_ty.is_external() && !coll_ty.is_error() {
                            self.err(
                                "SM003",
                                collection.span,
                                format!(
                                    "foreach collection must be an array, found {}",
                                    coll_ty.describe()
                                ),
                            );
                        }
                        Type::Any
                    });
                self.bind_loop_var(var, elem);
                self.loop_depth += 1;
                self.check_statement(body);
                self.loop_depth -= 1;
            }
            StmtKind::Switch(s) => {
                let scrut = self.check_expr(&s.scrutinee);
                self.loop_depth += 1;
                for arm in &s.arms {
                    if let Some(label) = &arm.label {
                        let lt = self.check_expr(label);
                        if !scrut.is_error()
                            && !scrut.is_external()
                            && !lt.is_error()
                            && !lt.is_external()
                            && self.conversion(&lt, &scrut).rank() == 255
                        {
                            self.err(
                                "SM026",
                                label.span,
                                format!(
                                    "switch case value of type {} is incompatible with scrutinee type {}",
                                    lt.describe(),
                                    scrut.describe()
                                ),
                            );
                        }
                    }
                    for s in &arm.stmts {
                        self.check_statement(s);
                    }
                }
                self.loop_depth -= 1;
            }
            StmtKind::Return { value } => {
                if let Some(v) = value {
                    let ty = self.check_expr_with_hint(v, self.ret_ty.clone().unwrap_or(Type::Any));
                    if let Some(ret) = &self.ret_ty {
                        if *ret == Type::Void {
                            self.err("SM051", v.span, "void function cannot return a value");
                        } else if !ret.is_error()
                            && !ret.is_external()
                            && !self.is_assignable(&ty, ret)
                        {
                            self.err(
                                "SM051",
                                v.span,
                                format!(
                                    "cannot return a value of type {} from a function returning {}",
                                    ty.describe(),
                                    ret.describe()
                                ),
                            );
                        }
                    }
                } else if let Some(ret) = &self.ret_ty {
                    if *ret != Type::Void && !ret.is_error() && !ret.is_external() {
                        self.err(
                            "SM051",
                            stmt.span,
                            format!("missing return value (expected {})", ret.describe()),
                        );
                    }
                }
            }
            StmtKind::Break | StmtKind::Continue => {
                if self.loop_depth == 0 {
                    self.err(
                        "SM052",
                        stmt.span,
                        "break/continue outside a loop or switch",
                    );
                }
            }
            StmtKind::Expr(e) => {
                let _ = self.check_expr(e);
            }
            StmtKind::Delete { target } => {
                let ty = self.check_expr(target);
                if !matches!(ty, Type::Class(_) | Type::Any) && !ty.is_external() && !ty.is_error()
                {
                    self.err(
                        "SM039",
                        target.span,
                        format!(
                            "delete requires a class-typed operand, found {}",
                            ty.describe()
                        ),
                    );
                }
            }
            StmtKind::Hook { target, value } => {
                let _ = self.check_expr(target);
                let _ = self.check_expr(value);
            }
            StmtKind::Error { .. } => {}
        }
    }

    pub(super) fn bind_loop_var(&mut self, var: &VarDecl, elem: Type) {
        let ty = match &var.kind {
            VarDeclKind::Typed(t) => self.resolve_type_ref(t, self.scope()),
            VarDeclKind::Define => elem,
        };
        let sym = Symbol {
            name: var.name.name.clone(),
            kind: SymbolKind::Variable,
            span: var.name.span,
            decl: var.name.id,
            visibility: Visibility::Public,
            ty,
            owner: None,
            flags: SymbolFlags {
                const_init: true,
                ..Default::default()
            },
        };
        let _ = self.program.tables.declare(self.scope(), sym);
    }

    pub(super) fn decl_type(&mut self, v: &VarDecl) -> Type {
        match &v.kind {
            VarDeclKind::Define => Type::Any,
            VarDeclKind::Typed(t) => self.resolve_type_ref(t, self.scope()),
        }
    }
}
