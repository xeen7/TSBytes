use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::type_checker::TypeChecker;

impl TypeChecker {
    pub(crate) fn check_class_decl(&mut self, name: &str, class_def: &ClassDef) {
        self.classes.insert(name.to_string(), class_def.clone());

        // Perform override modifier checks
        for member in &class_def.members {
            let (m_name, is_override) = match member {
                ClassMember::Field(f_name, _, _, mods) => (f_name, mods.is_override),
                ClassMember::Method(m_name, _, _, _, mods) => (m_name, mods.is_override),
                _ => continue,
            };

            if is_override {
                let mut resolved = false;
                if let Some(parent_name) = &class_def.extends {
                    if let Some(parent_def) = self.classes.get(parent_name) {
                        for p_member in &parent_def.members {
                            let p_name = match p_member {
                                ClassMember::Field(name, _, _, _) => name,
                                ClassMember::Method(name, _, _, _, _) => name,
                                _ => continue,
                            };
                            if p_name == m_name {
                                resolved = true;
                                break;
                            }
                        }
                    }
                }
                if !resolved {
                    self.errors.push(format!(
                        "Type error: Member '{}' of class '{}' is marked with 'override' but does not override any member in its base class.",
                        m_name, name
                    ));
                }
            }
        }

        self.env.bind(name.to_string(), Type::Class(name.to_string()));
        self.env.enter_scope();
        for member in &class_def.members {
            match member {
                ClassMember::Field(f_name, f_ty, init, _) => {
                    if let Some(expr) = init {
                        let init_ty = self.check_expr(expr);
                        if !init_ty.is_assignable_to(f_ty) {
                            self.errors.push(format!("Type error: Cannot assign type '{:?}' to field '{}' of type '{:?}'", init_ty, f_name, f_ty));
                        }
                    }
                }
                ClassMember::Method(_m_name, args, ret_type, body, _) => {
                    self.env.enter_scope();
                    for arg in args {
                        self.env.bind(arg.0.clone(), arg.1.clone());
                    }
                    let prev_ret = self.current_fn_return.clone();
                    self.current_fn_return = Some(ret_type.clone());
                    
                    for s in body { self.check_stmt(s); }
                    
                    self.current_fn_return = prev_ret;
                    self.env.exit_scope();
                }
                _ => {}
            }
        }
        self.env.exit_scope();
    }
}
