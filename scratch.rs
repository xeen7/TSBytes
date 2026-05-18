    pub fn emit_class(&mut self, class_path: &str, class_def: &crate::codegen::compiler::ir::ClassDef) -> Vec<u8> {
        self.class_path = Some(class_path.to_string());
        let this_class = self.cp.add_class(class_path).unwrap();
        
        let super_class_name = if let Some(extends) = &class_def.extends {
            extends.replace('.', "/")
        } else {
            "java/lang/Object".to_string()
        };
        let super_class = self.cp.add_class(&super_class_name).unwrap();
        
        let code_attr_name = self.cp.add_utf8("Code").unwrap();
        let mut fields = Vec::new();
        let mut has_constructor = false;
        
        for member in &class_def.members {
            match member {
                crate::codegen::compiler::ir::ClassMember::Field(name, _ty, _init, _mods) => {
                    let name_idx = self.cp.add_utf8(name).unwrap();
                    let desc_idx = self.cp.add_utf8("Ljava/lang/Object;").unwrap(); // always Object for now
                    fields.push(ristretto_classfile::attributes::Field { // WAIT! Field is in ristretto_classfile::Field
                        access_flags: ristretto_classfile::ClassAccessFlags::PUBLIC.bits() as u16, // wait!
                        name_index: name_idx,
                        descriptor_index: desc_idx,
                        attributes: vec![],
                    }); // need to know the type for Field
                }
                _ => {}
            }
        }
        Vec::new()
    }
