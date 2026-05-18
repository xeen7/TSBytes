/// Closure Lowering Pass
///
/// This HIR transformation pass runs BEFORE bytecode emission.
/// It finds arrow functions (lambdas) that capture outer variables,
/// and rewrites them into synthetic class declarations implementing
/// a functional interface pattern.
///
/// Example input HIR:
///   let x = 10;
///   let adder = (y: number) => x + y;
///
/// Transformed output HIR:
///   let x = 10;
///   // Synthetic class generated:
///   class Lambda$0 {
///       x: number;  // captured variable
///       constructor(x: number) { this.x = x; }
///       invoke(y: number): number { return this.x + y; }
///   }
///   let adder = new Lambda$0(x);

use crate::codegen::compiler::ir::*;
use std::collections::{HashSet, HashMap};
use std::cell::RefCell;

thread_local! {
    static GLOBAL_CLASSES: RefCell<Vec<(HirStmt, HashMap<String, Type>)>> = RefCell::new(Vec::new());
    static ENCLOSING_CLOSURE: RefCell<Vec<(String, HashSet<String>)>> = RefCell::new(Vec::new());
}

thread_local! {
    static LAMBDA_COUNTER: std::cell::Cell<u32> = std::cell::Cell::new(0);
}

fn next_lambda_name() -> String {
    LAMBDA_COUNTER.with(|c| {
        let n = c.get();
        c.set(n + 1);
        format!("Lambda${}", n)
    })
}

fn rewrite_generator_stmts(stmts: &[HirStmt]) -> Vec<HirStmt> {
    let mut out = Vec::new();
    for s in stmts {
        match s {
            HirStmt::Return(expr_opt) => {
                let val = expr_opt.clone().unwrap_or(HirExpr::NullLit);
                out.push(HirStmt::Expr(HirExpr::MethodCall(
                    Box::new(HirExpr::Var("__queue".to_string(), Type::Class("java/util/concurrent/LinkedBlockingQueue".to_string()))),
                    "put".to_string(),
                    vec![val],
                    Type::Void,
                )));
                out.push(HirStmt::Expr(HirExpr::MethodCall(
                    Box::new(HirExpr::Var("__queue".to_string(), Type::Class("java/util/concurrent/LinkedBlockingQueue".to_string()))),
                    "put".to_string(),
                    vec![HirExpr::GeneratorSentinel],
                    Type::Void,
                )));
                out.push(HirStmt::Return(None));
            }
            HirStmt::If(test, cons, alt) => {
                out.push(HirStmt::If(
                    test.clone(),
                    rewrite_generator_stmts(cons),
                    rewrite_generator_stmts(alt),
                ));
            }
            HirStmt::While(test, body) => {
                out.push(HirStmt::While(
                    test.clone(),
                    rewrite_generator_stmts(body),
                ));
            }
            other => out.push(other.clone()),
        }
    }
    out
}

fn lower_generator(body_stmts: &[HirStmt], _ret_type: &Type) -> Vec<HirStmt> {
    let mut thread_body = rewrite_generator_stmts(body_stmts);
    
    thread_body.push(HirStmt::Expr(HirExpr::MethodCall(
        Box::new(HirExpr::Var("__queue".to_string(), Type::Class("java/util/concurrent/LinkedBlockingQueue".to_string()))),
        "put".to_string(),
        vec![HirExpr::GeneratorSentinel],
        Type::Void,
    )));
    
    let init_queue = HirStmt::Let(
        "__queue".to_string(),
        Type::Class("java/util/concurrent/LinkedBlockingQueue".to_string()),
        HirExpr::New(
            "java/util/concurrent/LinkedBlockingQueue".to_string(),
            vec![],
            Type::Class("java/util/concurrent/LinkedBlockingQueue".to_string()),
        ),
    );
    
    let spawn_thread = HirStmt::Expr(HirExpr::VirtualThreadSpawn(Box::new(HirExpr::Arrow(
        vec![],
        Type::Void,
        thread_body,
        FnModifiers {
            is_async: false,
            is_generator: false,
            is_export: false,
        },
    ))));
    
    let return_generator = HirStmt::Return(Some(HirExpr::New(
        "com/tsdroid/runtime/TsGenerator".to_string(),
        vec![HirExpr::Var(
            "__queue".to_string(),
            Type::Class("java/util/concurrent/LinkedBlockingQueue".to_string()),
        )],
        Type::Class("com/tsdroid/runtime/TsGenerator".to_string()),
    )));
    
    vec![init_queue, spawn_thread, return_generator]
}

/// Run the closure lowering pass over a list of HIR statements.
/// Returns the transformed statements with arrow functions replaced
/// by synthetic class instantiations.
pub fn lower_closures(stmts: Vec<HirStmt>) -> Vec<HirStmt> {
    GLOBAL_CLASSES.with(|gc| gc.borrow_mut().clear());
    let mut output = Vec::new();
    let mut env_vars: HashMap<String, Type> = HashMap::new();

    for stmt in stmts {
        lower_stmt(stmt, &mut output, &mut env_vars);
    }
    
    let mut final_classes = Vec::new();
    while !GLOBAL_CLASSES.with(|gc| gc.borrow().is_empty()) {
        let current_batch = GLOBAL_CLASSES.with(|gc| gc.take());
        for (stmt, mut class_env) in current_batch {
            lower_stmt(stmt, &mut final_classes, &mut class_env);
        }
    }
    
    output.extend(final_classes);
    output
}

fn lower_stmt(stmt: HirStmt, output: &mut Vec<HirStmt>, env_vars: &mut HashMap<String, Type>) {
    match stmt {
        HirStmt::Let(ref name, ref ty, ref expr) => {
            env_vars.insert(name.clone(), ty.clone());
            let new_expr = lower_expr(expr.clone(), output, env_vars);
            output.push(HirStmt::Let(name.clone(), ty.clone(), new_expr));
        }
        HirStmt::Const(ref name, ref ty, ref expr) => {
            env_vars.insert(name.clone(), ty.clone());
            let new_expr = lower_expr(expr.clone(), output, env_vars);
            output.push(HirStmt::Const(name.clone(), ty.clone(), new_expr));
        }
        HirStmt::Assign(name, expr) => {
            let new_expr = lower_expr(expr, output, env_vars);
            output.push(HirStmt::Assign(name, new_expr));
        }
        HirStmt::Return(Some(expr)) => {
            let new_expr = lower_expr(expr, output, env_vars);
            output.push(HirStmt::Return(Some(new_expr)));
        }
        HirStmt::Expr(expr) => {
            let new_expr = lower_expr(expr, output, env_vars);
            output.push(HirStmt::Expr(new_expr));
        }
        HirStmt::If(test, cons, alt) => {
            let new_test = lower_expr(test, output, env_vars);
            let new_cons = lower_block(cons, env_vars);
            let new_alt = lower_block(alt, env_vars);
            output.push(HirStmt::If(new_test, new_cons, new_alt));
        }
        HirStmt::While(test, body) => {
            let new_test = lower_expr(test, output, env_vars);
            let new_body = lower_block(body, env_vars);
            output.push(HirStmt::While(new_test, new_body));
        }
        HirStmt::FnDecl(name, args, ret, body, mut modifiers) => {
            if modifiers.is_generator {
                modifiers.is_generator = false;
                modifiers.is_async = false;
                let lowered_body = lower_generator(&body, &ret);
                let mut inner_env = env_vars.clone();
                let fn_ty = Type::Function(args.iter().map(|arg| arg.ty.clone()).collect(), Box::new(ret.clone()));
                inner_env.insert(name.clone(), fn_ty);
                for arg in &args {
                    inner_env.insert(arg.name.clone(), arg.ty.clone());
                }
                let new_body = lower_block(lowered_body, &mut inner_env);
                output.push(HirStmt::FnDecl(name, args, ret, new_body, modifiers));
            } else {
                // Register function args as known variables
                let mut inner_env = env_vars.clone();
                let fn_ty = Type::Function(args.iter().map(|arg| arg.ty.clone()).collect(), Box::new(ret.clone()));
                inner_env.insert(name.clone(), fn_ty);
                for arg in &args {
                    inner_env.insert(arg.name.clone(), arg.ty.clone());
                }
                let new_body = lower_block(body, &mut inner_env);
                output.push(HirStmt::FnDecl(name, args, ret, new_body, modifiers));
            }
        }
        HirStmt::FieldAssign(obj, field, expr) => {
            let new_obj = lower_expr(obj.clone(), output, env_vars);
            let new_expr = lower_expr(expr.clone(), output, env_vars);
            output.push(HirStmt::FieldAssign(new_obj, field.clone(), new_expr));
        }
        HirStmt::DestructureObject(fields, rest_opt, source) => {
            let new_source = lower_expr(source.clone(), output, env_vars);
            for (prop_name, alias, _) in &fields {
                let local = alias.as_deref().unwrap_or(prop_name);
                if !local.is_empty() { env_vars.insert(local.to_string(), Type::Any); }
            }
            if let Some(rest) = &rest_opt {
                env_vars.insert(rest.clone(), Type::Any);
            }
            output.push(HirStmt::DestructureObject(fields.clone(), rest_opt, new_source));
        }
        HirStmt::DestructureArray(slots, rest_opt, source) => {
            let new_source = lower_expr(source.clone(), output, env_vars);
            for (name_opt, _) in &slots {
                if let Some(n) = name_opt { env_vars.insert(n.clone(), Type::Any); }
            }
            if let Some(rest) = &rest_opt {
                env_vars.insert(rest.clone(), Type::Any);
            }
            output.push(HirStmt::DestructureArray(slots.clone(), rest_opt, new_source));
        }
        HirStmt::ClassDecl(name, mut def) => {
            let mut cap_names = HashSet::new();
            for member in &def.members {
                if let ClassMember::Field(fname, _, _, _) = member {
                    cap_names.insert(fname.clone());
                }
            }
            ENCLOSING_CLOSURE.with(|ec| ec.borrow_mut().push((name.clone(), cap_names)));

            for member in &mut def.members {
                match member {
                    ClassMember::Method(_, args, _, body, _) => {
                        let mut inner_env = env_vars.clone();
                        for (arg_name, arg_ty) in args {
                            inner_env.insert(arg_name.clone(), arg_ty.clone());
                        }
                        *body = lower_block(body.clone(), &mut inner_env);
                    }
                    ClassMember::Constructor(args, body) => {
                        let mut inner_env = env_vars.clone();
                        for (arg_name, arg_ty) in args {
                            inner_env.insert(arg_name.clone(), arg_ty.clone());
                        }
                        *body = lower_block(body.clone(), &mut inner_env);
                    }
                    ClassMember::Getter(_, _, body, _) => {
                        let mut inner_env = env_vars.clone();
                        *body = lower_block(body.clone(), &mut inner_env);
                    }
                    ClassMember::Setter(_, param_name, param_ty, body, _) => {
                        let mut inner_env = env_vars.clone();
                        inner_env.insert(param_name.clone(), param_ty.clone());
                        *body = lower_block(body.clone(), &mut inner_env);
                    }
                    _ => {}
                }
            }

            ENCLOSING_CLOSURE.with(|ec| ec.borrow_mut().pop());
            output.push(HirStmt::ClassDecl(name, def));
        }
        HirStmt::CompoundAssign(name, op, expr) => {
            let new_expr = lower_expr(expr, output, env_vars);
            output.push(HirStmt::CompoundAssign(name, op, new_expr));
        }
        HirStmt::DoWhile(body, test) => {
            let new_body = lower_block(body, env_vars);
            let new_test = lower_expr(test, output, env_vars);
            output.push(HirStmt::DoWhile(new_body, new_test));
        }
        HirStmt::ForOf(var_name, var_ty, iterable, body) => {
            let new_iterable = lower_expr(iterable, output, env_vars);
            let mut inner_env = env_vars.clone();
            inner_env.insert(var_name.clone(), var_ty.clone());
            let new_body = lower_block(body, &mut inner_env);
            output.push(HirStmt::ForOf(var_name, var_ty, new_iterable, new_body));
        }
        HirStmt::ForAwaitOf(var_name, var_ty, iterable, body) => {
            let new_iterable = lower_expr(iterable, output, env_vars);
            let mut inner_env = env_vars.clone();
            inner_env.insert(var_name.clone(), var_ty.clone());
            let new_body = lower_block(body, &mut inner_env);
            output.push(HirStmt::ForAwaitOf(var_name, var_ty, new_iterable, new_body));
        }
        HirStmt::ForIn(var_name, obj, body) => {
            let new_obj = lower_expr(obj, output, env_vars);
            let mut inner_env = env_vars.clone();
            inner_env.insert(var_name.clone(), Type::StringTy);
            let new_body = lower_block(body, &mut inner_env);
            output.push(HirStmt::ForIn(var_name, new_obj, new_body));
        }
        HirStmt::Switch(expr, cases) => {
            let new_expr = lower_expr(expr, output, env_vars);
            let new_cases = cases.into_iter().map(|case| SwitchCase {
                test: case.test.map(|t| lower_expr(t, output, env_vars)),
                cons: lower_block(case.cons, env_vars),
            }).collect();
            output.push(HirStmt::Switch(new_expr, new_cases));
        }
        HirStmt::Labeled(label, stmt) => {
            let mut temp_out = Vec::new();
            lower_stmt(*stmt, &mut temp_out, env_vars);
            if let Some(new_stmt) = temp_out.pop() {
                output.push(HirStmt::Labeled(label, Box::new(new_stmt)));
            }
        }
        HirStmt::TryCatch(try_blk, catch_var, catch_blk, finally_blk) => {
            let new_try = lower_block(try_blk, env_vars);
            let mut catch_env = env_vars.clone();
            if let Some(ref cv) = catch_var {
                catch_env.insert(cv.clone(), Type::Any);
            }
            let new_catch = lower_block(catch_blk, &mut catch_env);
            let new_finally = lower_block(finally_blk, env_vars);
            output.push(HirStmt::TryCatch(new_try, catch_var, new_catch, new_finally));
        }
        HirStmt::Throw(expr) => {
            let new_expr = lower_expr(expr, output, env_vars);
            output.push(HirStmt::Throw(new_expr));
        }
        // Pass through everything else unchanged
        other => output.push(other),
    }
}

fn lower_block(stmts: Vec<HirStmt>, env_vars: &mut HashMap<String, Type>) -> Vec<HirStmt> {
    let mut output = Vec::new();
    for stmt in stmts {
        lower_stmt(stmt, &mut output, env_vars);
    }
    output
}

/// Recursively walk an expression, replacing Arrow functions with
/// synthetic class instantiations.
fn lower_expr(
    expr: HirExpr,
    output: &mut Vec<HirStmt>,
    env_vars: &HashMap<String, Type>,
) -> HirExpr {
    match expr {
        HirExpr::Arrow(params, ret_type, body, mut modifiers) => {
            if modifiers.is_generator {
                modifiers.is_generator = false;
                modifiers.is_async = false;
                let lowered_body = lower_generator(&body, &ret_type);
                let lowered_arrow = HirExpr::Arrow(params, ret_type, lowered_body, modifiers);
                return lower_expr(lowered_arrow, output, env_vars);
            }
            // 1. Find captured variables: variables used in body that are NOT parameters
            let param_names: HashSet<String> = params.iter().map(|arg| arg.name.clone()).collect();
            let used_vars = collect_used_vars_in_stmts(&body);
            let captured: Vec<(String, Type)> = used_vars
                .iter()
                .filter(|name| !param_names.contains(*name) && env_vars.contains_key(*name))
                .map(|name| {
                    let ty = env_vars.get(name).cloned().unwrap_or(Type::Any);
                    (name.clone(), ty)
                })
                .collect();

            // 2. Generate a synthetic class name
            let class_name = next_lambda_name();


            // 3. Build constructor that takes captured vars
            let mut ctor_body = Vec::new();
            for (cap_name, cap_ty) in &captured {
                ctor_body.push(HirStmt::FieldAssign(
                    HirExpr::This(Type::Class(class_name.clone())),
                    cap_name.clone(),
                    HirExpr::Var(cap_name.clone(), cap_ty.clone()),
                ));
            }

            // 4. Rewrite body: replace captured var references with this.field
            let rewritten_body = rewrite_captured_refs(&body, &captured, &class_name);

            // 5. Build class members
            let mut members = Vec::new();

            // Fields for captured variables
            for (cap_name, cap_ty) in &captured {
                members.push(ClassMember::Field(
                    cap_name.clone(),
                    cap_ty.clone(),
                    None,
                    FieldModifiers::default(),
                ));
            }

            // Constructor
            members.push(ClassMember::Constructor(
                captured.clone(),
                ctor_body,
            ));

            let is_supplier = params.is_empty();
            let implements = if is_supplier {
                let bridge_body = vec![HirStmt::Return(Some(HirExpr::MethodCall(
                    Box::new(HirExpr::This(Type::Any)),
                    "invoke".to_string(),
                    vec![],
                    Type::Any,
                )))];
                members.push(ClassMember::Method(
                    "get".to_string(),
                    vec![],
                    Type::Any,
                    bridge_body,
                    MethodModifiers::default(),
                ));
                vec!["java/util/function/Supplier".to_string()]
            } else {
                vec![]
            };

            // main method with the lambda's body (always named "invoke" for runtime reflection)
            members.push(ClassMember::Method(
                "invoke".to_string(),
                params.clone().into_iter().map(|a| (a.name, a.ty)).collect(),
                ret_type.clone(),
                rewritten_body,
                MethodModifiers::default(),
            ));

            // 6. Emit the synthetic class declaration
            GLOBAL_CLASSES.with(|gc| gc.borrow_mut().push((
                HirStmt::ClassDecl(
                    class_name.clone(),
                    ClassDef {
                        extends: None,
                        implements,
                        is_abstract: false,
                        members,
                    },
                ),
                env_vars.clone(),
            )));

            // 7. Return a `new Lambda$N(captured_var1, captured_var2, ...)`
            let ctor_args: Vec<HirExpr> = ENCLOSING_CLOSURE.with(|ec| {
                let enclosing = ec.borrow();
                let last = enclosing.last();
                captured
                    .iter()
                    .map(|(name, ty)| {
                        let raw_var = HirExpr::Var(name.clone(), ty.clone());
                        if let Some((enc_class, enc_caps)) = last {
                            if enc_caps.contains(name) {
                                HirExpr::FieldGet(
                                    Box::new(HirExpr::This(Type::Class(enc_class.clone()))),
                                    name.clone(),
                                    ty.clone(),
                                )
                            } else {
                                raw_var
                            }
                        } else {
                            raw_var
                        }
                    })
                    .collect()
            });
            HirExpr::New(class_name.clone(), ctor_args, Type::Class(class_name))
        }
        // Recursively lower sub-expressions
        HirExpr::BinOp(op, left, right, ty) => {
            let l = lower_expr(*left, output, env_vars);
            let r = lower_expr(*right, output, env_vars);
            HirExpr::BinOp(op, Box::new(l), Box::new(r), ty)
        }
        HirExpr::UnaryOp(op, expr, ty) => {
            let e = lower_expr(*expr, output, env_vars);
            HirExpr::UnaryOp(op, Box::new(e), ty)
        }
        HirExpr::Ternary(cond, then_e, else_e, ty) => {
            let c = lower_expr(*cond, output, env_vars);
            let t = lower_expr(*then_e, output, env_vars);
            let e = lower_expr(*else_e, output, env_vars);
            HirExpr::Ternary(Box::new(c), Box::new(t), Box::new(e), ty)
        }
        HirExpr::Call(name, args, ty) => {
            let new_args: Vec<HirExpr> = args.into_iter().map(|a| lower_expr(a, output, env_vars)).collect();
            HirExpr::Call(name, new_args, ty)
        }
        HirExpr::MethodCall(obj, method, args, ty) => {
            let new_obj = lower_expr(*obj, output, env_vars);
            let new_args: Vec<HirExpr> = args.into_iter().map(|a| lower_expr(a, output, env_vars)).collect();
            HirExpr::MethodCall(Box::new(new_obj), method, new_args, ty)
        }
        HirExpr::SuperCall(name, args, ty) => {
            let new_args: Vec<HirExpr> = args.into_iter().map(|a| lower_expr(a, output, env_vars)).collect();
            HirExpr::SuperCall(name, new_args, ty)
        }
        HirExpr::New(class, args, ty) => {
            let new_args: Vec<HirExpr> = args.into_iter().map(|a| lower_expr(a, output, env_vars)).collect();
            HirExpr::New(class, new_args, ty)
        }
        HirExpr::FieldGet(obj, field, ty) => {
            let new_obj = lower_expr(*obj, output, env_vars);
            HirExpr::FieldGet(Box::new(new_obj), field, ty)
        }
        HirExpr::FieldSet(obj, field, val, ty) => {
            let new_obj = lower_expr(*obj, output, env_vars);
            let new_val = lower_expr(*val, output, env_vars);
            HirExpr::FieldSet(Box::new(new_obj), field, Box::new(new_val), ty)
        }
        HirExpr::IndexGet(obj, idx, ty) => {
            let new_obj = lower_expr(*obj, output, env_vars);
            let new_idx = lower_expr(*idx, output, env_vars);
            HirExpr::IndexGet(Box::new(new_obj), Box::new(new_idx), ty)
        }
        HirExpr::IndexSet(obj, idx, val, ty) => {
            let new_obj = lower_expr(*obj, output, env_vars);
            let new_idx = lower_expr(*idx, output, env_vars);
            let new_val = lower_expr(*val, output, env_vars);
            HirExpr::IndexSet(Box::new(new_obj), Box::new(new_idx), Box::new(new_val), ty)
        }
        HirExpr::OptionalChain(obj, field, ty) => {
            let new_obj = lower_expr(*obj, output, env_vars);
            HirExpr::OptionalChain(Box::new(new_obj), field, ty)
        }
        HirExpr::OptionalCall(callee, args, ty) => {
            let new_callee = lower_expr(*callee, output, env_vars);
            let new_args: Vec<HirExpr> = args.into_iter().map(|a| lower_expr(a, output, env_vars)).collect();
            HirExpr::OptionalCall(Box::new(new_callee), new_args, ty)
        }
        HirExpr::DynamicCall(callee, args, ty) => {
            let new_callee = lower_expr(*callee, output, env_vars);
            let new_args: Vec<HirExpr> = args.into_iter().map(|a| lower_expr(a, output, env_vars)).collect();
            HirExpr::DynamicCall(Box::new(new_callee), new_args, ty)
        }
        HirExpr::ArrayLit(elems, ty) => {
            let new_elems: Vec<HirExpr> = elems.into_iter().map(|e| lower_expr(e, output, env_vars)).collect();
            HirExpr::ArrayLit(new_elems, ty)
        }
        HirExpr::ObjectLit(props, ty) => {
            let new_props: Vec<ObjectProp> = props.into_iter().map(|p| {
                match p {
                    ObjectProp::KeyValue(k, v) => ObjectProp::KeyValue(k, lower_expr(v, output, env_vars)),
                    ObjectProp::Computed(k, v) => ObjectProp::Computed(lower_expr(k, output, env_vars), lower_expr(v, output, env_vars)),
                    ObjectProp::Spread(e) => ObjectProp::Spread(lower_expr(e, output, env_vars)),
                    ObjectProp::Getter(name, arrow) => {
                        let rewritten = rewrite_this_expr(&arrow);
                        ObjectProp::Getter(name, lower_expr(rewritten, output, env_vars))
                    }
                    ObjectProp::Setter(name, arrow) => {
                        let rewritten = rewrite_this_expr(&arrow);
                        ObjectProp::Setter(name, lower_expr(rewritten, output, env_vars))
                    }
                    other => other,
                }
            }).collect();
            HirExpr::ObjectLit(new_props, ty)
        }
        HirExpr::DeleteProp(obj, prop) => {
            let new_obj = lower_expr(*obj, output, env_vars);
            HirExpr::DeleteProp(Box::new(new_obj), prop)
        }
        HirExpr::ObjectMethod(name, args, ty) => {
            let new_args: Vec<HirExpr> = args.into_iter().map(|a| lower_expr(a, output, env_vars)).collect();
            HirExpr::ObjectMethod(name, new_args, ty)
        }
        HirExpr::Cast(expr, ty) => {
            let new_expr = lower_expr(*expr, output, env_vars);
            HirExpr::Cast(Box::new(new_expr), ty)
        }
        HirExpr::InstanceOf(expr, class_name) => {
            let new_expr = lower_expr(*expr, output, env_vars);
            HirExpr::InstanceOf(Box::new(new_expr), class_name)
        }
        HirExpr::TemplateLit(exprs) => {
            let new_exprs: Vec<HirExpr> = exprs.into_iter().map(|e| lower_expr(e, output, env_vars)).collect();
            HirExpr::TemplateLit(new_exprs)
        }
        HirExpr::Spread(expr) => {
            let new_expr = lower_expr(*expr, output, env_vars);
            HirExpr::Spread(Box::new(new_expr))
        }
        HirExpr::Await(expr, ty) => {
            let new_expr = lower_expr(*expr, output, env_vars);
            HirExpr::Await(Box::new(new_expr), ty)
        }
        HirExpr::Seq(exprs, ty) => {
            let new_exprs: Vec<HirExpr> = exprs.into_iter().map(|e| lower_expr(e, output, env_vars)).collect();
            HirExpr::Seq(new_exprs, ty)
        }
        HirExpr::VirtualThreadSpawn(closure) => {
            let new_closure = lower_expr(*closure, output, env_vars);
            HirExpr::VirtualThreadSpawn(Box::new(new_closure))
        }
        HirExpr::LogicalAnd(left, right, ty) => {
            let l = lower_expr(*left, output, env_vars);
            let r = lower_expr(*right, output, env_vars);
            HirExpr::LogicalAnd(Box::new(l), Box::new(r), ty)
        }
        HirExpr::LogicalOr(left, right, ty) => {
            let l = lower_expr(*left, output, env_vars);
            let r = lower_expr(*right, output, env_vars);
            HirExpr::LogicalOr(Box::new(l), Box::new(r), ty)
        }
        HirExpr::ArrayLen(expr) => {
            let new_expr = lower_expr(*expr, output, env_vars);
            HirExpr::ArrayLen(Box::new(new_expr))
        }
        HirExpr::ArrayMethod(expr, method, args, ty) => {
            let new_expr = lower_expr(*expr, output, env_vars);
            let new_args: Vec<HirExpr> = args.into_iter().map(|a| lower_expr(a, output, env_vars)).collect();
            HirExpr::ArrayMethod(Box::new(new_expr), method, new_args, ty)
        }
        HirExpr::DynamicImport(expr) => {
            let new_expr = lower_expr(*expr, output, env_vars);
            HirExpr::DynamicImport(Box::new(new_expr))
        }
        HirExpr::TaggedTemplate(tag, quasis, exprs) => {
            let new_tag = lower_expr(*tag, output, env_vars);
            let new_quasis: Vec<HirExpr> = quasis.into_iter().map(|q| lower_expr(q, output, env_vars)).collect();
            let new_exprs: Vec<HirExpr> = exprs.into_iter().map(|e| lower_expr(e, output, env_vars)).collect();
            HirExpr::TaggedTemplate(Box::new(new_tag), new_quasis, new_exprs)
        }
        HirExpr::Yield(expr, is_delegate) => {
            let new_expr = expr.map(|e| Box::new(lower_expr(*e, output, env_vars)));
            HirExpr::Yield(new_expr, is_delegate)
        }
        // Leaf expressions pass through unchanged
        other => other,
    }
}

/// Collect all variable names used in a list of statements.
fn collect_used_vars_in_stmts(stmts: &[HirStmt]) -> HashSet<String> {
    let mut vars = HashSet::new();
    for stmt in stmts {
        collect_used_vars_in_stmt(stmt, &mut vars);
    }
    vars
}

fn collect_used_vars_in_stmt(stmt: &HirStmt, vars: &mut HashSet<String>) {
    match stmt {
        HirStmt::Let(_, _, expr) | HirStmt::Const(_, _, expr) => collect_used_vars(expr, vars),
        HirStmt::Assign(name, expr) => {
            vars.insert(name.clone());
            collect_used_vars(expr, vars);
        }
        HirStmt::CompoundAssign(name, _, expr) => {
            vars.insert(name.clone());
            collect_used_vars(expr, vars);
        }
        HirStmt::FieldAssign(obj, _, expr) => {
            collect_used_vars(obj, vars);
            collect_used_vars(expr, vars);
        }
        HirStmt::Return(Some(expr)) => collect_used_vars(expr, vars),
        HirStmt::Expr(expr) => collect_used_vars(expr, vars),
        HirStmt::If(test, cons, alt) => {
            collect_used_vars(test, vars);
            for s in cons { collect_used_vars_in_stmt(s, vars); }
            for s in alt { collect_used_vars_in_stmt(s, vars); }
        }
        HirStmt::While(test, body) => {
            collect_used_vars(test, vars);
            for s in body { collect_used_vars_in_stmt(s, vars); }
        }
        HirStmt::DoWhile(body, test) => {
            collect_used_vars(test, vars);
            for s in body { collect_used_vars_in_stmt(s, vars); }
        }
        HirStmt::ForOf(_, _, iterable, body) | HirStmt::ForAwaitOf(_, _, iterable, body) => {
            collect_used_vars(iterable, vars);
            for s in body { collect_used_vars_in_stmt(s, vars); }
        }
        HirStmt::ForIn(_, iterable, body) => {
            collect_used_vars(iterable, vars);
            for s in body { collect_used_vars_in_stmt(s, vars); }
        }
        HirStmt::TryCatch(try_block, _, catch_block, finally_block) => {
            for s in try_block { collect_used_vars_in_stmt(s, vars); }
            for s in catch_block { collect_used_vars_in_stmt(s, vars); }
            for s in finally_block { collect_used_vars_in_stmt(s, vars); }
        }
        HirStmt::Throw(expr) => collect_used_vars(expr, vars),
        HirStmt::DestructureObject(_, _, source) => collect_used_vars(source, vars),
        HirStmt::DestructureArray(_, _, source) => collect_used_vars(source, vars),
        _ => {}
    }
}

fn collect_used_vars(expr: &HirExpr, vars: &mut HashSet<String>) {
    match expr {
        HirExpr::Var(name, _) => { vars.insert(name.clone()); }
        HirExpr::BinOp(_, l, r, _) => { collect_used_vars(l, vars); collect_used_vars(r, vars); }
        HirExpr::UnaryOp(_, a, _) => collect_used_vars(a, vars),
        HirExpr::Call(_, args, _) => { for a in args { collect_used_vars(a, vars); } }
        HirExpr::MethodCall(obj, _, args, _) => {
            collect_used_vars(obj, vars);
            for a in args { collect_used_vars(a, vars); }
        }
        HirExpr::Ternary(c, t, e, _) => {
            collect_used_vars(c, vars);
            collect_used_vars(t, vars);
            collect_used_vars(e, vars);
        }
        HirExpr::FieldGet(obj, _, _) => collect_used_vars(obj, vars),
        HirExpr::FieldSet(obj, _, val, _) => { collect_used_vars(obj, vars); collect_used_vars(val, vars); }
        HirExpr::IndexGet(obj, idx, _) => { collect_used_vars(obj, vars); collect_used_vars(idx, vars); }
        HirExpr::IndexSet(obj, idx, val, _) => { collect_used_vars(obj, vars); collect_used_vars(idx, vars); collect_used_vars(val, vars); }
        HirExpr::ArrayLit(elems, _) => { for e in elems { collect_used_vars(e, vars); } }
        HirExpr::Arrow(_, _, body, _) => { collect_used_vars_in_stmts(body).into_iter().for_each(|v| { vars.insert(v); }); }
        HirExpr::ObjectLit(props, _) => {
            for prop in props {
                match prop {
                    ObjectProp::KeyValue(_, v) => collect_used_vars(v, vars),
                    ObjectProp::Computed(k, v) => { collect_used_vars(k, vars); collect_used_vars(v, vars); }
                    ObjectProp::Spread(e) => collect_used_vars(e, vars),
                    _ => {}
                }
            }
        }
        HirExpr::DeleteProp(obj, _) => collect_used_vars(obj, vars),
        HirExpr::ObjectMethod(_, args, _) => { for a in args { collect_used_vars(a, vars); } }
        HirExpr::OptionalChain(expr, _, _) => collect_used_vars(expr, vars),
        HirExpr::OptionalCall(callee, args, _) => {
            collect_used_vars(callee, vars);
            for a in args {
                collect_used_vars(a, vars);
            }
        }
        HirExpr::DynamicCall(callee, args, _) => {
            collect_used_vars(callee, vars);
            for a in args {
                collect_used_vars(a, vars);
            }
        }
        HirExpr::VirtualThreadSpawn(closure) => collect_used_vars(closure, vars),
        HirExpr::DynamicImport(path) => collect_used_vars(path, vars),
        HirExpr::TaggedTemplate(tag, quasis, exprs) => {
            collect_used_vars(tag, vars);
            for q in quasis { collect_used_vars(q, vars); }
            for e in exprs { collect_used_vars(e, vars); }
        }
        HirExpr::Yield(arg, _) => {
            if let Some(a) = arg {
                collect_used_vars(a, vars);
            }
        }
        HirExpr::Seq(exprs, _) => { for e in exprs { collect_used_vars(e, vars); } }
        _ => {}
    }
}

/// Rewrite variable references inside a lambda body to use `this.field`
/// for any variable that was captured from the outer scope.
fn rewrite_captured_refs(
    stmts: &[HirStmt],
    captured: &[(String, Type)],
    class_name: &str,
) -> Vec<HirStmt> {
    let cap_names: HashSet<&str> = captured.iter().map(|(n, _)| n.as_str()).collect();
    stmts.iter().map(|s| rewrite_stmt(s, &cap_names, class_name)).collect()
}

/// Rewrite `this` expressions into `$this` variable references
/// Used for getters/setters where the target object is passed as the first argument.
fn rewrite_this_to_var(stmts: &[HirStmt]) -> Vec<HirStmt> {
    stmts.iter().map(|s| rewrite_this_stmt(s)).collect()
}

fn rewrite_this_stmt(stmt: &HirStmt) -> HirStmt {
    match stmt {
        HirStmt::Let(n, t, e) => HirStmt::Let(n.clone(), t.clone(), rewrite_this_expr(e)),
        HirStmt::Const(n, t, e) => HirStmt::Const(n.clone(), t.clone(), rewrite_this_expr(e)),
        HirStmt::Expr(e) => HirStmt::Expr(rewrite_this_expr(e)),
        HirStmt::Return(e) => HirStmt::Return(e.as_ref().map(|x| rewrite_this_expr(x))),
        HirStmt::If(c, t, e) => HirStmt::If(
            rewrite_this_expr(c),
            rewrite_this_to_var(t),
            rewrite_this_to_var(e),
        ),
        HirStmt::While(c, b) => HirStmt::While(rewrite_this_expr(c), rewrite_this_to_var(b)),
        HirStmt::Assign(n, e) => HirStmt::Assign(n.clone(), rewrite_this_expr(e)),
        HirStmt::FieldAssign(obj, f, v) => HirStmt::FieldAssign(rewrite_this_expr(obj), f.clone(), rewrite_this_expr(v)),
        HirStmt::DoWhile(b, c) => HirStmt::DoWhile(rewrite_this_to_var(b), rewrite_this_expr(c)),
        HirStmt::ForOf(n, t, e, b) => HirStmt::ForOf(n.clone(), t.clone(), rewrite_this_expr(e), rewrite_this_to_var(b)),
        HirStmt::ForAwaitOf(n, t, e, b) => HirStmt::ForAwaitOf(n.clone(), t.clone(), rewrite_this_expr(e), rewrite_this_to_var(b)),
        HirStmt::ForIn(n, e, b) => HirStmt::ForIn(n.clone(), rewrite_this_expr(e), rewrite_this_to_var(b)),
        HirStmt::DestructureObject(p, rest_opt, e) => HirStmt::DestructureObject(p.clone(), rest_opt.clone(), rewrite_this_expr(e)),
        HirStmt::DestructureArray(p, rest_opt, e) => HirStmt::DestructureArray(p.clone(), rest_opt.clone(), rewrite_this_expr(e)),
        _ => stmt.clone(),
    }
}

fn rewrite_this_expr(expr: &HirExpr) -> HirExpr {
    match expr {
        HirExpr::This(_) => HirExpr::Var("$this".to_string(), Type::Any),
        HirExpr::BinOp(op, l, r, ty) => HirExpr::BinOp(op.clone(), Box::new(rewrite_this_expr(l)), Box::new(rewrite_this_expr(r)), ty.clone()),
        HirExpr::UnaryOp(op, a, ty) => HirExpr::UnaryOp(op.clone(), Box::new(rewrite_this_expr(a)), ty.clone()),
        HirExpr::Call(n, args, ty) => HirExpr::Call(n.clone(), args.iter().map(rewrite_this_expr).collect(), ty.clone()),
        HirExpr::MethodCall(obj, m, args, ty) => HirExpr::MethodCall(Box::new(rewrite_this_expr(obj)), m.clone(), args.iter().map(rewrite_this_expr).collect(), ty.clone()),
        HirExpr::Ternary(c, t, e, ty) => HirExpr::Ternary(Box::new(rewrite_this_expr(c)), Box::new(rewrite_this_expr(t)), Box::new(rewrite_this_expr(e)), ty.clone()),
        HirExpr::FieldGet(obj, f, ty) => HirExpr::FieldGet(Box::new(rewrite_this_expr(obj)), f.clone(), ty.clone()),
        HirExpr::FieldSet(obj, f, v, ty) => HirExpr::FieldSet(Box::new(rewrite_this_expr(obj)), f.clone(), Box::new(rewrite_this_expr(v)), ty.clone()),
        HirExpr::IndexGet(obj, idx, ty) => HirExpr::IndexGet(Box::new(rewrite_this_expr(obj)), Box::new(rewrite_this_expr(idx)), ty.clone()),
        HirExpr::IndexSet(obj, idx, v, ty) => HirExpr::IndexSet(Box::new(rewrite_this_expr(obj)), Box::new(rewrite_this_expr(idx)), Box::new(rewrite_this_expr(v)), ty.clone()),
        HirExpr::ArrayLit(elems, ty) => HirExpr::ArrayLit(elems.iter().map(rewrite_this_expr).collect(), ty.clone()),
        HirExpr::ObjectLit(props, ty) => {
            let new_props = props.iter().map(|p| match p {
                ObjectProp::KeyValue(k, v) => ObjectProp::KeyValue(k.clone(), rewrite_this_expr(v)),
                ObjectProp::Computed(k, v) => ObjectProp::Computed(rewrite_this_expr(k), rewrite_this_expr(v)),
                ObjectProp::Spread(e) => ObjectProp::Spread(rewrite_this_expr(e)),
                _ => p.clone(),
            }).collect();
            HirExpr::ObjectLit(new_props, ty.clone())
        }
        HirExpr::Arrow(params, ty, body, mods) => {
            HirExpr::Arrow(params.clone(), ty.clone(), rewrite_this_to_var(body), mods.clone())
        }
        HirExpr::OptionalCall(callee, args, ty) => {
            HirExpr::OptionalCall(
                Box::new(rewrite_this_expr(callee)),
                args.iter().map(rewrite_this_expr).collect(),
                ty.clone(),
            )
        }
        HirExpr::DynamicCall(callee, args, ty) => {
            HirExpr::DynamicCall(
                Box::new(rewrite_this_expr(callee)),
                args.iter().map(rewrite_this_expr).collect(),
                ty.clone(),
            )
        }
        HirExpr::VirtualThreadSpawn(closure) => {
            HirExpr::VirtualThreadSpawn(Box::new(rewrite_this_expr(closure)))
        }
        HirExpr::DynamicImport(path) => {
            HirExpr::DynamicImport(Box::new(rewrite_this_expr(path)))
        }
        HirExpr::TaggedTemplate(tag, quasis, exprs) => {
            HirExpr::TaggedTemplate(
                Box::new(rewrite_this_expr(tag)),
                quasis.iter().map(rewrite_this_expr).collect(),
                exprs.iter().map(rewrite_this_expr).collect(),
            )
        }
        HirExpr::Yield(arg, delegate) => {
            HirExpr::Yield(
                arg.as_ref().map(|a| Box::new(rewrite_this_expr(a))),
                *delegate,
            )
        }
        HirExpr::Seq(exprs, ty) => {
            HirExpr::Seq(exprs.iter().map(rewrite_this_expr).collect(), ty.clone())
        }
        _ => expr.clone(),
    }
}

fn rewrite_stmt(stmt: &HirStmt, cap_names: &HashSet<&str>, class_name: &str) -> HirStmt {
    match stmt {
        HirStmt::Let(name, ty, expr) => {
            HirStmt::Let(name.clone(), ty.clone(), rewrite_expr(expr, cap_names, class_name))
        }
        HirStmt::Const(name, ty, expr) => {
            HirStmt::Const(name.clone(), ty.clone(), rewrite_expr(expr, cap_names, class_name))
        }
        HirStmt::Return(Some(expr)) => {
            HirStmt::Return(Some(rewrite_expr(expr, cap_names, class_name)))
        }
        HirStmt::Return(None) => HirStmt::Return(None),
        HirStmt::Expr(expr) => {
            HirStmt::Expr(rewrite_expr(expr, cap_names, class_name))
        }
        HirStmt::Assign(name, expr) => {
            if cap_names.contains(name.as_str()) {
                let this_expr = HirExpr::This(Type::Class(class_name.to_string()));
                HirStmt::FieldAssign(this_expr, name.clone(), rewrite_expr(expr, cap_names, class_name))
            } else {
                HirStmt::Assign(name.clone(), rewrite_expr(expr, cap_names, class_name))
            }
        }
        HirStmt::CompoundAssign(name, op, expr) => {
            if cap_names.contains(name.as_str()) {
                let this_expr = HirExpr::This(Type::Class(class_name.to_string()));
                let var_ty = Type::Double;
                let field_get = HirExpr::FieldGet(Box::new(this_expr.clone()), name.clone(), var_ty.clone());
                let bin_op_kind = match op {
                    AssignOp::AddAssign => BinOp::Add,
                    AssignOp::SubAssign => BinOp::Sub,
                    AssignOp::MulAssign => BinOp::Mul,
                    AssignOp::DivAssign => BinOp::Div,
                    _ => BinOp::Add,
                };
                let new_expr = HirExpr::BinOp(
                    bin_op_kind,
                    Box::new(field_get),
                    Box::new(rewrite_expr(expr, cap_names, class_name)),
                    var_ty,
                );
                HirStmt::FieldAssign(this_expr, name.clone(), new_expr)
            } else {
                HirStmt::CompoundAssign(name.clone(), op.clone(), rewrite_expr(expr, cap_names, class_name))
            }
        }
        HirStmt::FieldAssign(obj, field, expr) => {
            HirStmt::FieldAssign(
                rewrite_expr(obj, cap_names, class_name),
                field.clone(),
                rewrite_expr(expr, cap_names, class_name),
            )
        }
        HirStmt::If(test, cons, alt) => {
            HirStmt::If(
                rewrite_expr(test, cap_names, class_name),
                cons.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
                alt.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
            )
        }
        HirStmt::While(test, body) => {
            HirStmt::While(
                rewrite_expr(test, cap_names, class_name),
                body.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
            )
        }
        HirStmt::DoWhile(body, test) => {
            HirStmt::DoWhile(
                body.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
                rewrite_expr(test, cap_names, class_name),
            )
        }
        HirStmt::ForOf(var_name, var_ty, iterable, body) => {
            HirStmt::ForOf(
                var_name.clone(),
                var_ty.clone(),
                rewrite_expr(iterable, cap_names, class_name),
                body.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
            )
        }
        HirStmt::ForAwaitOf(var_name, var_ty, iterable, body) => {
            HirStmt::ForAwaitOf(
                var_name.clone(),
                var_ty.clone(),
                rewrite_expr(iterable, cap_names, class_name),
                body.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
            )
        }
        HirStmt::ForIn(var_name, iterable, body) => {
            HirStmt::ForIn(
                var_name.clone(),
                rewrite_expr(iterable, cap_names, class_name),
                body.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
            )
        }
        HirStmt::TryCatch(try_block, param, catch_block, finally_block) => {
            HirStmt::TryCatch(
                try_block.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
                param.clone(),
                catch_block.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
                finally_block.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
            )
        }
        HirStmt::Throw(expr) => {
            HirStmt::Throw(rewrite_expr(expr, cap_names, class_name))
        }
        other => other.clone(),
    }
}

fn rewrite_expr(expr: &HirExpr, cap_names: &HashSet<&str>, class_name: &str) -> HirExpr {
    match expr {
        HirExpr::Var(name, ty) if cap_names.contains(name.as_str()) => {
            // Replace `x` with `this.x`
            HirExpr::FieldGet(
                Box::new(HirExpr::This(Type::Class(class_name.to_string()))),
                name.clone(),
                ty.clone(),
            )
        }
        HirExpr::BinOp(op, l, r, ty) => {
            HirExpr::BinOp(
                op.clone(),
                Box::new(rewrite_expr(l, cap_names, class_name)),
                Box::new(rewrite_expr(r, cap_names, class_name)),
                ty.clone(),
            )
        }
        HirExpr::Call(name, args, ty) => {
            let new_args: Vec<HirExpr> = args.iter().map(|a| rewrite_expr(a, cap_names, class_name)).collect();
            HirExpr::Call(name.clone(), new_args, ty.clone())
        }
        HirExpr::MethodCall(obj, method, args, ty) => {
            let new_obj = rewrite_expr(obj, cap_names, class_name);
            let new_args: Vec<HirExpr> = args.iter().map(|a| rewrite_expr(a, cap_names, class_name)).collect();
            HirExpr::MethodCall(Box::new(new_obj), method.clone(), new_args, ty.clone())
        }
        HirExpr::IndexGet(obj, idx, ty) => {
            HirExpr::IndexGet(
                Box::new(rewrite_expr(obj, cap_names, class_name)),
                Box::new(rewrite_expr(idx, cap_names, class_name)),
                ty.clone(),
            )
        }
        HirExpr::IndexSet(obj, idx, val, ty) => {
            HirExpr::IndexSet(
                Box::new(rewrite_expr(obj, cap_names, class_name)),
                Box::new(rewrite_expr(idx, cap_names, class_name)),
                Box::new(rewrite_expr(val, cap_names, class_name)),
                ty.clone(),
            )
        }
        HirExpr::New(cname, args, ty) => {
            let new_args: Vec<HirExpr> = args.iter().map(|a| rewrite_expr(a, cap_names, class_name)).collect();
            HirExpr::New(cname.clone(), new_args, ty.clone())
        }
        HirExpr::UnaryOp(op, expr, ty) => {
            HirExpr::UnaryOp(op.clone(), Box::new(rewrite_expr(expr, cap_names, class_name)), ty.clone())
        }
        HirExpr::Ternary(cond, cons, alt, ty) => {
            HirExpr::Ternary(
                Box::new(rewrite_expr(cond, cap_names, class_name)),
                Box::new(rewrite_expr(cons, cap_names, class_name)),
                Box::new(rewrite_expr(alt, cap_names, class_name)),
                ty.clone(),
            )
        }
        HirExpr::ObjectLit(props, ty) => {
            let new_props: Vec<ObjectProp> = props
                .iter()
                .map(|p| match p {
                    ObjectProp::KeyValue(k, v) => ObjectProp::KeyValue(k.clone(), rewrite_expr(v, cap_names, class_name)),
                    ObjectProp::Computed(k, v) => ObjectProp::Computed(rewrite_expr(k, cap_names, class_name), rewrite_expr(v, cap_names, class_name)),
                    ObjectProp::Method(name, args, rty, body) => ObjectProp::Method(
                        name.clone(),
                        args.clone(),
                        rty.clone(),
                        body.iter().map(|s| rewrite_stmt(s, cap_names, class_name)).collect(),
                    ),
                    ObjectProp::Getter(name, expr) => ObjectProp::Getter(name.clone(), rewrite_expr(expr, cap_names, class_name)),
                    ObjectProp::Setter(name, expr) => ObjectProp::Setter(name.clone(), rewrite_expr(expr, cap_names, class_name)),
                    ObjectProp::Spread(expr) => ObjectProp::Spread(rewrite_expr(expr, cap_names, class_name)),
                })
                .collect();
            HirExpr::ObjectLit(new_props, ty.clone())
        }
        HirExpr::FieldGet(obj, field, ty) => {
            HirExpr::FieldGet(Box::new(rewrite_expr(obj, cap_names, class_name)), field.clone(), ty.clone())
        }
        HirExpr::FieldSet(obj, field, val, ty) => {
            HirExpr::FieldSet(
                Box::new(rewrite_expr(obj, cap_names, class_name)),
                field.clone(),
                Box::new(rewrite_expr(val, cap_names, class_name)),
                ty.clone(),
            )
        }
        HirExpr::DeleteProp(obj, prop) => {
            HirExpr::DeleteProp(Box::new(rewrite_expr(obj, cap_names, class_name)), prop.clone())
        }
        HirExpr::ObjectMethod(name, args, ty) => {
            let new_args: Vec<HirExpr> = args.iter().map(|a| rewrite_expr(a, cap_names, class_name)).collect();
            HirExpr::ObjectMethod(name.clone(), new_args, ty.clone())
        }
        HirExpr::Arrow(params, ty, body, mods) => {
            HirExpr::Arrow(params.clone(), ty.clone(), rewrite_this_to_var(&body), mods.clone())
        }
        HirExpr::Cast(expr, ty) => {
            HirExpr::Cast(Box::new(rewrite_expr(expr, cap_names, class_name)), ty.clone())
        }
        HirExpr::InstanceOf(expr, cname) => {
            HirExpr::InstanceOf(Box::new(rewrite_expr(expr, cap_names, class_name)), cname.clone())
        }
        HirExpr::TemplateLit(exprs) => {
            let new_exprs: Vec<HirExpr> = exprs.iter().map(|e| rewrite_expr(e, cap_names, class_name)).collect();
            HirExpr::TemplateLit(new_exprs)
        }
        HirExpr::Spread(expr) => {
            HirExpr::Spread(Box::new(rewrite_expr(expr, cap_names, class_name)))
        }
        HirExpr::Await(expr, ty) => {
            HirExpr::Await(Box::new(rewrite_expr(expr, cap_names, class_name)), ty.clone())
        }
        HirExpr::LogicalAnd(l, r, ty) => {
            HirExpr::LogicalAnd(
                Box::new(rewrite_expr(l, cap_names, class_name)),
                Box::new(rewrite_expr(r, cap_names, class_name)),
                ty.clone(),
            )
        }
        HirExpr::LogicalOr(l, r, ty) => {
            HirExpr::LogicalOr(
                Box::new(rewrite_expr(l, cap_names, class_name)),
                Box::new(rewrite_expr(r, cap_names, class_name)),
                ty.clone(),
            )
        }
        HirExpr::ArrayLen(expr) => {
            HirExpr::ArrayLen(Box::new(rewrite_expr(expr, cap_names, class_name)))
        }
        HirExpr::ArrayMethod(expr, method, args, ty) => {
            let new_expr = rewrite_expr(expr, cap_names, class_name);
            let new_args: Vec<HirExpr> = args.iter().map(|a| rewrite_expr(a, cap_names, class_name)).collect();
            HirExpr::ArrayMethod(Box::new(new_expr), method.clone(), new_args, ty.clone())
        }
        HirExpr::OptionalChain(expr, field, ty) => {
            HirExpr::OptionalChain(Box::new(rewrite_expr(expr, cap_names, class_name)), field.clone(), ty.clone())
        }
        HirExpr::OptionalCall(callee, args, ty) => {
            let new_callee = rewrite_expr(callee, cap_names, class_name);
            let new_args: Vec<HirExpr> = args.iter().map(|a| rewrite_expr(a, cap_names, class_name)).collect();
            HirExpr::OptionalCall(Box::new(new_callee), new_args, ty.clone())
        }
        HirExpr::DynamicCall(callee, args, ty) => {
            let new_callee = rewrite_expr(callee, cap_names, class_name);
            let new_args: Vec<HirExpr> = args.iter().map(|a| rewrite_expr(a, cap_names, class_name)).collect();
            HirExpr::DynamicCall(Box::new(new_callee), new_args, ty.clone())
        }
        HirExpr::SuperCall(name, args, ty) => {
            let new_args: Vec<HirExpr> = args.iter().map(|a| rewrite_expr(a, cap_names, class_name)).collect();
            HirExpr::SuperCall(name.clone(), new_args, ty.clone())
        }
        HirExpr::VirtualThreadSpawn(closure) => {
            HirExpr::VirtualThreadSpawn(Box::new(rewrite_expr(closure, cap_names, class_name)))
        }
        HirExpr::DynamicImport(path) => {
            HirExpr::DynamicImport(Box::new(rewrite_expr(path, cap_names, class_name)))
        }
        HirExpr::TaggedTemplate(tag, quasis, exprs) => {
            let new_tag = rewrite_expr(tag, cap_names, class_name);
            let new_quasis: Vec<HirExpr> = quasis.iter().map(|q| rewrite_expr(q, cap_names, class_name)).collect();
            let new_exprs: Vec<HirExpr> = exprs.iter().map(|e| rewrite_expr(e, cap_names, class_name)).collect();
            HirExpr::TaggedTemplate(Box::new(new_tag), new_quasis, new_exprs)
        }
        HirExpr::Yield(arg, delegate) => {
            let new_arg = arg.as_ref().map(|a| Box::new(rewrite_expr(a, cap_names, class_name)));
            HirExpr::Yield(new_arg, *delegate)
        }
        HirExpr::Seq(exprs, ty) => {
            let new_exprs: Vec<HirExpr> = exprs.iter().map(|e| rewrite_expr(e, cap_names, class_name)).collect();
            HirExpr::Seq(new_exprs, ty.clone())
        }
        other => other.clone(),
    }
}
