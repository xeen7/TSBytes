use ristretto_classfile::{ClassFile, ConstantPool, Method, Field, FieldType, Version, ClassAccessFlags, MethodAccessFlags, FieldAccessFlags, attributes::{Instruction, Attribute, StackFrame, VerificationType, CodeException}};
use crate::codegen::compiler::ir::*;
use std::collections::HashMap;

pub mod expr;
pub mod stmt;
pub mod method;

fn get_or_add_class(cp: &mut ConstantPool, class_name: &str) -> u16 {
    use ristretto_classfile::Constant;
    let mut max_idx = 0;
    for i in 1..u16::MAX {
        match cp.get(i) {
            Some(Constant::Class(utf8_idx)) => {
                if let Some(Constant::Utf8(utf8_val)) = cp.get(*utf8_idx) {
                    if utf8_val.as_str() == class_name {
                        return i;
                    }
                }
                max_idx = i;
            }
            Some(_) => {
                max_idx = i;
            }
            None => {
                if i > max_idx + 10 {
                    break;
                }
            }
        }
    }
    cp.add_class(class_name).unwrap_or(0)
}

/// Generate StackMapTable frames for a list of instructions.
/// Uses FullFrame for all branch targets, tracking locals from store instructions.
pub fn generate_stack_map_table(cp: &mut ConstantPool, code: &[Instruction], exceptions: &[CodeException], initial_locals: Vec<VerificationType>) -> Vec<Attribute> {
    use std::collections::BTreeSet;
    use crate::codegen::compiler::bytecode_emitter::method::inst_size;
    use ristretto_classfile::attributes::VerificationType;

    if code.is_empty() {
        return vec![];
    }
    use ristretto_classfile::Constant;
    if let Some(Constant::FieldRef { class_index: _, name_and_type_index }) = cp.get(1) {
        if let Some(Constant::NameAndType { name_index: _, descriptor_index }) = cp.get(*name_and_type_index) {
            if let Some(Constant::Utf8(desc_str)) = cp.get(*descriptor_index) {
                let _s: &str = desc_str.as_str();
            }
        }
    }

    // Compute byte offset for each instruction index
    let mut byte_offsets: Vec<usize> = Vec::with_capacity(code.len());
    let mut offset = 0usize;
    for inst in code {
        byte_offsets.push(offset);
        offset += inst_size(inst);
    }

    let is_terminal = |inst: &Instruction| -> bool {
        matches!(inst,
            Instruction::Areturn | Instruction::Return |
            Instruction::Ireturn | Instruction::Dreturn |
            Instruction::Freturn | Instruction::Lreturn |
            Instruction::Athrow | Instruction::Goto(_)
        )
    };

    let merge_types = |v1: &VerificationType, v2: &VerificationType, object_class_idx: u16, cp: &ConstantPool| -> VerificationType {
        if v1 == v2 {
            return v1.clone();
        }
        match (v1, v2) {
            (VerificationType::Null, VerificationType::Object { cpool_index }) => {
                VerificationType::Object { cpool_index: *cpool_index }
            }
            (VerificationType::Object { cpool_index }, VerificationType::Null) => {
                VerificationType::Object { cpool_index: *cpool_index }
            }
            (VerificationType::Object { cpool_index: cp1 }, VerificationType::Object { cpool_index: cp2 }) => {
                let name1 = if let Some(Constant::Class(name_index)) = cp.get(*cp1) {
                    if let Some(Constant::Utf8(utf8)) = cp.get(*name_index) {
                        Some(utf8.as_str().to_string())
                    } else { None }
                } else { None };
                
                let name2 = if let Some(Constant::Class(name_index)) = cp.get(*cp2) {
                    if let Some(Constant::Utf8(utf8)) = cp.get(*name_index) {
                        Some(utf8.as_str().to_string())
                    } else { None }
                } else { None };
                
                if name1.is_some() && name1 == name2 {
                    VerificationType::Object { cpool_index: *cp1 }
                } else {
                    VerificationType::Object { cpool_index: object_class_idx }
                }
            }
            _ => VerificationType::Top,
        }
    };

    let mut post_terminal: BTreeSet<usize> = BTreeSet::new();
    for i in 0..code.len() - 1 {
        if is_terminal(&code[i]) {
            post_terminal.insert(i + 1);
        }
    }

    // Every instruction following a terminal instruction is a new basic block boundary.
    // We must generate a stack map frame for it to satisfy the JVM verifier.
    let mut target_indices = BTreeSet::new();
    for &idx in &post_terminal {
        if idx < byte_offsets.len() {
            target_indices.insert(idx);
        }
    }

    // Find all branch target instruction indices
    for inst in code.iter() {
        let target_idx = match inst {
            Instruction::Ifeq(t) | Instruction::Ifne(t) |
            Instruction::Iflt(t) | Instruction::Ifle(t) |
            Instruction::Ifgt(t) | Instruction::Ifge(t) |
            Instruction::Goto(t) | Instruction::Ifnull(t) |
            Instruction::Ifnonnull(t) => Some(*t as usize),
            _ => None,
        };
        if let Some(idx) = target_idx {
            if idx < byte_offsets.len() {
                target_indices.insert(idx);
            }
        }
    }

    // Include exception handler targets and record their corresponding try block start PC
    let mut handler_targets = std::collections::HashSet::new();
    let mut handler_to_start = std::collections::HashMap::new();
    for exc in exceptions {
        let idx = exc.handler_pc as usize;
        let start = exc.start_pc as usize;
        if idx < byte_offsets.len() {
            target_indices.insert(idx);
            handler_targets.insert(idx);
            handler_to_start.entry(idx)
                .and_modify(|existing| *existing = std::cmp::min(*existing, start))
                .or_insert(start);
        }
    }

    if target_indices.is_empty() {
        return vec![];
    }

    let object_class_idx = cp.add_class("java/lang/Object").unwrap_or(0);

    // Iterative dataflow analysis to compute correct verification types at each instruction index
    let n_inst = code.len();
    type LocalsState = std::collections::BTreeMap<u16, VerificationType>;
    let mut in_locals: Vec<Option<LocalsState>> = vec![None; n_inst];
    let mut out_locals: Vec<Option<LocalsState>> = vec![None; n_inst];
    
    let mut in_stack: Vec<Option<Vec<VerificationType>>> = vec![None; n_inst];
    let mut out_stack: Vec<Option<Vec<VerificationType>>> = vec![None; n_inst];

    // Initialize entry state
    let mut entry_locals = std::collections::BTreeMap::new();
    for (i, vtype) in initial_locals.iter().enumerate() {
        entry_locals.insert(i as u16, vtype.clone());
    }
    in_locals[0] = Some(entry_locals);
    in_stack[0] = Some(vec![]);

    let mut changed = true;
    let mut iterations = 0;
    while changed && iterations < 100 {
        changed = false;
        iterations += 1;

        for i in 0..n_inst {
            // 1. Compute out_locals[i] from in_locals[i]
            if let Some(curr_in) = &in_locals[i] {
                let mut curr_out = curr_in.clone();
                match &code[i] {
                    Instruction::Astore(s) => {
                        let ty = in_stack[i].as_ref()
                            .and_then(|st| st.last().cloned())
                            .unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out.insert(*s as u16, ty);
                    }
                    Instruction::Astore_0 => {
                        let ty = in_stack[i].as_ref().and_then(|st| st.last().cloned()).unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out.insert(0, ty);
                    }
                    Instruction::Astore_1 => {
                        let ty = in_stack[i].as_ref().and_then(|st| st.last().cloned()).unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out.insert(1, ty);
                    }
                    Instruction::Astore_2 => {
                        let ty = in_stack[i].as_ref().and_then(|st| st.last().cloned()).unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out.insert(2, ty);
                    }
                    Instruction::Astore_3 => {
                        let ty = in_stack[i].as_ref().and_then(|st| st.last().cloned()).unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out.insert(3, ty);
                    }
                    Instruction::Istore(s) => { curr_out.insert(*s as u16, VerificationType::Integer); }
                    Instruction::Dstore(s) => {
                        curr_out.insert(*s as u16, VerificationType::Double);
                        curr_out.insert(*s as u16 + 1, VerificationType::Top);
                    }
                    Instruction::Fstore(s) => { curr_out.insert(*s as u16, VerificationType::Float); }
                    Instruction::Lstore(s) => {
                        curr_out.insert(*s as u16, VerificationType::Long);
                        curr_out.insert(*s as u16 + 1, VerificationType::Top);
                    }
                    _ => {}
                }
                if out_locals[i].as_ref() != Some(&curr_out) {
                    out_locals[i] = Some(curr_out);
                    changed = true;
                }
            }

            // 1b. Compute out_stack[i] from in_stack[i]
            if let Some(curr_in_stack) = &in_stack[i] {
                let mut curr_out_stack = curr_in_stack.clone();
                match &code[i] {
                    // Push Object
                    Instruction::Aconst_null => {
                        curr_out_stack.push(VerificationType::Object { cpool_index: object_class_idx });
                    }
                    Instruction::Ldc_w(idx) => {
                        let mut is_string = false;
                        if let Some(c) = cp.get(*idx) {
                            match c {
                                Constant::String(_) => { is_string = true; }
                                _ => {}
                            }
                        }
                        if is_string {
                            let string_class_idx = get_or_add_class(cp, "java/lang/String");
                            curr_out_stack.push(VerificationType::Object { cpool_index: string_class_idx });
                        } else {
                            curr_out_stack.push(VerificationType::Object { cpool_index: object_class_idx });
                        }
                    }
                    Instruction::Ldc(idx) => {
                        let mut is_string = false;
                        if let Some(c) = cp.get(*idx as u16) {
                            match c {
                                Constant::String(_) => { is_string = true; }
                                _ => {}
                            }
                        }
                        if is_string {
                            let string_class_idx = get_or_add_class(cp, "java/lang/String");
                            curr_out_stack.push(VerificationType::Object { cpool_index: string_class_idx });
                        } else {
                            curr_out_stack.push(VerificationType::Object { cpool_index: object_class_idx });
                        }
                    }
                    Instruction::New(idx) => {
                        curr_out_stack.push(VerificationType::Object { cpool_index: *idx });
                    }
                    Instruction::Checkcast(idx) => {
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Object { cpool_index: *idx });
                    }
                    Instruction::Aload(s) => {
                        let ty = in_locals[i].as_ref().and_then(|l| l.get(&(*s as u16)).cloned()).unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out_stack.push(ty);
                    }
                    Instruction::Aload_0 => {
                        let ty = in_locals[i].as_ref().and_then(|l| l.get(&0).cloned()).unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out_stack.push(ty);
                    }
                    Instruction::Aload_1 => {
                        let ty = in_locals[i].as_ref().and_then(|l| l.get(&1).cloned()).unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out_stack.push(ty);
                    }
                    Instruction::Aload_2 => {
                        let ty = in_locals[i].as_ref().and_then(|l| l.get(&2).cloned()).unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out_stack.push(ty);
                    }
                    Instruction::Aload_3 => {
                        let ty = in_locals[i].as_ref().and_then(|l| l.get(&3).cloned()).unwrap_or_else(|| VerificationType::Object { cpool_index: object_class_idx });
                        curr_out_stack.push(ty);
                    }
                    
                    // Push Double (takes 2 slots: Double, Top)
                    Instruction::Dload(_) | Instruction::Dload_0 | Instruction::Dload_1 | Instruction::Dload_2 | Instruction::Dload_3 |
                    Instruction::Dconst_0 | Instruction::Dconst_1 | Instruction::Ldc2_w(_) => {
                        curr_out_stack.push(VerificationType::Double);
                        curr_out_stack.push(VerificationType::Top);
                    }
                    
                    // Push Integer
                    Instruction::Iconst_0 | Instruction::Iconst_1 | Instruction::Iconst_m1 | 
                    Instruction::Iconst_2 | Instruction::Iconst_3 | Instruction::Iconst_4 | Instruction::Iconst_5 |
                    Instruction::Bipush(_) | Instruction::Sipush(_) |
                    Instruction::Iload(_) | Instruction::Iload_0 | Instruction::Iload_1 | Instruction::Iload_2 | Instruction::Iload_3 => {
                        curr_out_stack.push(VerificationType::Integer);
                    }
                    
                    Instruction::Instanceof(_) => {
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Integer);
                    }
                    
                    // Pop 1 slot
                    Instruction::Astore(_) | Instruction::Astore_0 | Instruction::Astore_1 | Instruction::Astore_2 | Instruction::Astore_3 |
                    Instruction::Istore(_) | Instruction::Istore_0 | Instruction::Istore_1 | Instruction::Istore_2 | Instruction::Istore_3 |
                    Instruction::Pop | Instruction::Ifeq(_) | Instruction::Ifne(_) | Instruction::Iflt(_) | Instruction::Ifle(_) |
                    Instruction::Ifgt(_) | Instruction::Ifge(_) | Instruction::Ifnull(_) | Instruction::Ifnonnull(_) |
                    Instruction::Ireturn | Instruction::Areturn | Instruction::Athrow => {
                        curr_out_stack.pop();
                    }
                    
                    // Pop 2 slots
                    Instruction::Dstore(_) | Instruction::Dstore_0 | Instruction::Dstore_1 | Instruction::Dstore_2 | Instruction::Dstore_3 |
                    Instruction::Pop2 | Instruction::Dreturn | Instruction::If_icmpeq(_) | Instruction::If_icmpne(_) |
                    Instruction::If_icmplt(_) | Instruction::If_icmple(_) | Instruction::If_icmpgt(_) | Instruction::If_icmpge(_) |
                    Instruction::If_acmpeq(_) | Instruction::If_acmpne(_) => {
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                    }
                    
                    // Dup
                    Instruction::Dup => {
                        if let Some(top) = curr_in_stack.last() {
                            curr_out_stack.push(top.clone());
                        }
                    }
                    
                    // Dup2
                    Instruction::Dup2 => {
                        let len = curr_in_stack.len();
                        if len >= 2 {
                            let t1 = curr_in_stack[len - 2].clone();
                            let t2 = curr_in_stack[len - 1].clone();
                            curr_out_stack.push(t1);
                            curr_out_stack.push(t2);
                        }
                    }
                    
                    // Swap
                    Instruction::Swap => {
                        let len = curr_out_stack.len();
                        if len >= 2 {
                            curr_out_stack.swap(len - 1, len - 2);
                        }
                    }
                    
                    // Array
                    Instruction::Anewarray(_) | Instruction::Newarray(_) => {
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Object { cpool_index: object_class_idx });
                    }
                    Instruction::Aaload => {
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Object { cpool_index: object_class_idx });
                    }
                    Instruction::Aastore => {
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                    }
                    
                    // Conversions
                    Instruction::I2d => {
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Double);
                        curr_out_stack.push(VerificationType::Top);
                    }
                    Instruction::D2i => {
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Integer);
                    }
                    
                    // Math
                    Instruction::Iadd | Instruction::Isub | Instruction::Imul | Instruction::Idiv | Instruction::Irem |
                    Instruction::Iand | Instruction::Ior | Instruction::Ixor | Instruction::Ishl | Instruction::Ishr | Instruction::Iushr => {
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Integer);
                    }
                    Instruction::Dcmpg | Instruction::Dcmpl => {
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Integer);
                    }
                    Instruction::Dadd | Instruction::Dsub | Instruction::Dmul | Instruction::Ddiv | Instruction::Drem => {
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Double);
                        curr_out_stack.push(VerificationType::Top);
                    }
                    
                    // Fields
                    Instruction::Getfield(_) => {
                        curr_out_stack.pop();
                        curr_out_stack.push(VerificationType::Object { cpool_index: object_class_idx });
                    }
                    Instruction::Putfield(_) => {
                        curr_out_stack.pop();
                        curr_out_stack.pop();
                    }
                    Instruction::Getstatic(_) => {
                        curr_out_stack.push(VerificationType::Object { cpool_index: object_class_idx });
                    }
                    Instruction::Putstatic(_) => {
                        curr_out_stack.pop();
                    }
                    
                    // Invocations
                    Instruction::Invokevirtual(idx) | Instruction::Invokespecial(idx) |
                    Instruction::Invokestatic(idx) | Instruction::Invokeinterface(idx, _) => {
                        let mut num_args = 0;
                        let mut returns_double = false;
                        let mut returns_void = false;
                        let mut returns_object = false;
                        let mut object_class_name = String::new();
                        let is_static = matches!(code[i], Instruction::Invokestatic(_));
                        
                        if let Some(Constant::MethodRef { class_index: _, name_and_type_index }) |
                           Some(Constant::InterfaceMethodRef { class_index: _, name_and_type_index }) = cp.get(*idx) {
                            if let Some(Constant::NameAndType { name_index: name_idx, descriptor_index }) = cp.get(*name_and_type_index) {
                                let _method_name = if let Some(Constant::Utf8(name_str)) = cp.get(*name_idx) { name_str.as_str().to_string() } else { "".to_string() };
                                if let Some(Constant::Utf8(desc_str)) = cp.get(*descriptor_index) {
                                    let s = desc_str.as_str();

                                    if let Some(start) = s.find('(') {
                                        if let Some(end) = s.find(')') {
                                            let args_part = &s[start+1..end];
                                            let ret_part = &s[end+1..];
                                            
                                            let mut chars = args_part.chars().peekable();
                                            while let Some(c) = chars.next() {
                                                match c {
                                                    'D' | 'J' => num_args += 2,
                                                    'L' => {
                                                        num_args += 1;
                                                        while let Some(next_c) = chars.next() {
                                                            if next_c == ';' { break; }
                                                        }
                                                    }
                                                    '[' => {
                                                        num_args += 1;
                                                        while let Some(&next_c) = chars.peek() {
                                                            if next_c == '[' {
                                                                chars.next();
                                                            } else if next_c == 'L' {
                                                                chars.next();
                                                                while let Some(next_c2) = chars.next() {
                                                                    if next_c2 == ';' { break; }
                                                                }
                                                                break;
                                                            } else {
                                                                chars.next();
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    _ => num_args += 1,
                                                }
                                            }
                                            
                                            if ret_part.starts_with('D') || ret_part.starts_with('J') {
                                                returns_double = true;
                                            } else if ret_part.starts_with('V') {
                                                returns_void = true;
                                            } else if ret_part.starts_with('L') {
                                                returns_object = true;
                                                let mut cn = &ret_part[1..];
                                                if cn.ends_with(';') {
                                                    cn = &cn[..cn.len() - 1];
                                                }
                                                object_class_name = cn.to_string();
                                            } else if ret_part.starts_with('[') {
                                                returns_object = true;
                                                object_class_name = ret_part.to_string();
                                            }

                                        }
                                    }
                                }
                            }
                        }
                        
                        for _ in 0..num_args {
                            curr_out_stack.pop();
                        }
                        if !is_static {
                            curr_out_stack.pop();
                        }
                        if !returns_void {
                            if returns_double {
                                curr_out_stack.push(VerificationType::Double);
                                curr_out_stack.push(VerificationType::Top);
                            } else if returns_object {
                                let cpool_index = get_or_add_class(cp, &object_class_name);
                                curr_out_stack.push(VerificationType::Object { cpool_index });
                            } else {
                                curr_out_stack.push(VerificationType::Object { cpool_index: object_class_idx });
                            }
                        }
                    }
                    
                    Instruction::Return => {
                        curr_out_stack.clear();
                    }
                    
                    _ => {}
                }
                if out_stack[i].as_ref() != Some(&curr_out_stack) {
                    out_stack[i] = Some(curr_out_stack);
                    changed = true;
                }
            }

            // 2. Propagate out_locals[i] and out_stack[i] to successors
            if let Some(curr_out) = &out_locals[i] {
                // Find control flow successors
                let mut successors = Vec::new();
                match &code[i] {
                    Instruction::Goto(t) => {
                        successors.push(*t as usize);
                    }
                    Instruction::Ireturn | Instruction::Lreturn | Instruction::Freturn |
                    Instruction::Dreturn | Instruction::Areturn | Instruction::Return |
                    Instruction::Athrow => {
                        // Terminal, no successors
                    }
                    Instruction::Ifeq(t) | Instruction::Ifne(t) | Instruction::Iflt(t) |
                    Instruction::Ifle(t) | Instruction::Ifgt(t) | Instruction::Ifge(t) |
                    Instruction::Ifnull(t) | Instruction::Ifnonnull(t) | Instruction::If_icmpeq(t) |
                    Instruction::If_icmpne(t) | Instruction::If_icmplt(t) | Instruction::If_icmple(t) |
                    Instruction::If_icmpgt(t) | Instruction::If_icmpge(t) | Instruction::If_acmpeq(t) |
                    Instruction::If_acmpne(t) => {
                        successors.push(*t as usize);
                        if i + 1 < n_inst {
                            successors.push(i + 1);
                        }
                    }
                    _ => {
                        if i + 1 < n_inst {
                            successors.push(i + 1);
                        }
                    }
                }

                // Propagate locals to successors
                for &succ in &successors {
                    if succ < n_inst {
                        let merged = match &in_locals[succ] {
                            None => curr_out.clone(),
                            Some(existing) => {
                                let mut m = std::collections::BTreeMap::new();
                                for (&k, v) in existing.iter() {
                                    if let Some(v_out) = curr_out.get(&k) {
                                        m.insert(k, merge_types(v, v_out, object_class_idx, cp));
                                    } else {
                                        m.insert(k, VerificationType::Top);
                                    }
                                }
                                m
                            }
                        };
                        if in_locals[succ].as_ref() != Some(&merged) {
                            in_locals[succ] = Some(merged);
                            changed = true;
                        }
                    }
                }
            }

            if let Some(curr_out_stack) = &out_stack[i] {
                // Find control flow successors
                let mut successors = Vec::new();
                match &code[i] {
                    Instruction::Goto(t) => {
                        successors.push(*t as usize);
                    }
                    Instruction::Ireturn | Instruction::Lreturn | Instruction::Freturn |
                    Instruction::Dreturn | Instruction::Areturn | Instruction::Return |
                    Instruction::Athrow => {
                        // Terminal, no successors
                    }
                    Instruction::Ifeq(t) | Instruction::Ifne(t) | Instruction::Iflt(t) |
                    Instruction::Ifle(t) | Instruction::Ifgt(t) | Instruction::Ifge(t) |
                    Instruction::Ifnull(t) | Instruction::Ifnonnull(t) | Instruction::If_icmpeq(t) |
                    Instruction::If_icmpne(t) | Instruction::If_icmplt(t) | Instruction::If_icmple(t) |
                    Instruction::If_icmpgt(t) | Instruction::If_icmpge(t) | Instruction::If_acmpeq(t) |
                    Instruction::If_acmpne(t) => {
                        successors.push(*t as usize);
                        if i + 1 < n_inst {
                            successors.push(i + 1);
                        }
                    }
                    _ => {
                        if i + 1 < n_inst {
                            successors.push(i + 1);
                        }
                    }
                }

                // Propagate stack to successors
                for &succ in &successors {
                    if succ < n_inst {
                        let merged = match &in_stack[succ] {
                            None => curr_out_stack.clone(),
                            Some(existing) => {
                                let mut m = Vec::new();
                                if existing.len() == curr_out_stack.len() {
                                    for (v1, v2) in existing.iter().zip(curr_out_stack.iter()) {
                                        m.push(merge_types(v1, v2, object_class_idx, cp));
                                    }
                                }
                                m
                            }
                        };
                        if in_stack[succ].as_ref() != Some(&merged) {
                            in_stack[succ] = Some(merged);
                            changed = true;
                        }
                    }
                }
            }

            // Propagate to any applicable exception handlers
            for exc in exceptions.iter() {
                let start_idx = exc.start_pc as usize;
                let end_idx = exc.end_pc as usize;
                let handler_idx = exc.handler_pc as usize;

                if i >= start_idx && i < end_idx {
                    if let Some(curr_in) = &in_locals[i] {
                        let merged = match &in_locals[handler_idx] {
                            None => curr_in.clone(),
                            Some(existing) => {
                                let mut m = std::collections::BTreeMap::new();
                                for (&k, v) in existing.iter() {
                                    if let Some(v_in) = curr_in.get(&k) {
                                        m.insert(k, merge_types(v, v_in, object_class_idx, cp));
                                    } else {
                                        m.insert(k, VerificationType::Top);
                                    }
                                }
                                m
                            }
                        };
                        if in_locals[handler_idx].as_ref() != Some(&merged) {
                            in_locals[handler_idx] = Some(merged);
                            changed = true;
                        }
                    }

                    // stack propagation (cleared and throwable pushed)
                    let throwable_idx = get_or_add_class(cp, "java/lang/Throwable");
                    let handler_stack = vec![VerificationType::Object { cpool_index: throwable_idx }];
                    
                    let merged = match &in_stack[handler_idx] {
                        None => handler_stack,
                        Some(existing) => {
                            let mut m = Vec::new();
                            if existing.len() == handler_stack.len() {
                                for (v1, v2) in existing.iter().zip(handler_stack.iter()) {
                                    m.push(merge_types(v1, v2, object_class_idx, cp));
                                }
                            }
                            m
                        }
                    };
                    if in_stack[handler_idx].as_ref() != Some(&merged) {
                        in_stack[handler_idx] = Some(merged);
                        changed = true;
                    }
                }
            }
        }
    }

    // Build FullFrame for each target
    let mut frames: Vec<StackFrame> = Vec::new();
    let mut prev_byte: Option<usize> = None;
    for &idx in &target_indices {

        let target_byte = byte_offsets[idx];
        let delta = match prev_byte {
            None => target_byte,
            Some(prev) => {
                if target_byte <= prev {
                    continue;
                }
                target_byte - prev - 1
            }
        };
        prev_byte = Some(target_byte);

        let locals_idx = if let Some(&start_idx) = handler_to_start.get(&idx) {
            start_idx
        } else {
            idx
        };
        
        let locals_state = in_locals[locals_idx].clone().unwrap_or_else(|| {
            let mut fallback = None;
            for prev in (0..locals_idx).rev() {
                if let Some(state) = &in_locals[prev] {
                    fallback = Some(state.clone());
                    break;
                }
            }
            fallback.unwrap_or_else(|| {
                let mut entry = std::collections::BTreeMap::new();
                for (i, vtype) in initial_locals.iter().enumerate() {
                    entry.insert(i as u16, vtype.clone());
                }
                entry
            })
        });

        let mut locals = Vec::new();
        if !locals_state.is_empty() {
            let max_slot = *locals_state.keys().last().unwrap();
            let mut skip_next = false;
            for s in 0..=max_slot {
                if skip_next {
                    skip_next = false;
                    continue;
                }
                let v = locals_state.get(&s).cloned().unwrap_or(VerificationType::Top);
                locals.push(v.clone());
                if matches!(v, VerificationType::Double | VerificationType::Long) {
                    skip_next = true;
                }
            }
        }

        let mut stack = vec![];
        if handler_targets.contains(&idx) {
            let throwable_idx = get_or_add_class(cp, "java/lang/Throwable");
            stack.push(VerificationType::Object { cpool_index: throwable_idx });
        } else if let Some(target_stack) = &in_stack[idx] {
            stack = target_stack.clone();
        } else {
            if !post_terminal.contains(&idx) && idx > 0 {
                match &code[idx - 1] {
                    Instruction::Iconst_0 | Instruction::Iconst_1 | Instruction::Iconst_m1 | 
                    Instruction::Iconst_2 | Instruction::Iconst_3 | Instruction::Iconst_4 | Instruction::Iconst_5 |
                    Instruction::Bipush(_) | Instruction::Sipush(_) |
                    Instruction::Iload(_) | Instruction::Iload_0 | Instruction::Iload_1 | Instruction::Iload_2 | Instruction::Iload_3 => {
                        stack.push(VerificationType::Integer);
                    },
                    Instruction::Aload(_) | Instruction::Aload_0 | Instruction::Aload_1 | Instruction::Aload_2 | Instruction::Aload_3 |
                    Instruction::Aconst_null | Instruction::Ldc_w(_) => {
                        stack.push(VerificationType::Object { cpool_index: object_class_idx });
                    },
                    _ => {}
                }
            }
        }

        frames.push(StackFrame::FullFrame {
            frame_type: 255,
            offset_delta: delta as u16,
            locals,
            stack,
        });
    }

    if frames.is_empty() {
        return vec![];
    }

    let smt_name = match cp.add_utf8("StackMapTable") {
        Ok(idx) => idx,
        Err(e) => {
            println!("StackMapTable error: {:?}", e);
            panic!("ConstantPool overflow! constant pool size is extremely large.");
        }
    };
    vec![Attribute::StackMapTable {
        name_index: smt_name,
        frames,
    }]
}

pub fn resolve_jump_targets(code: &mut [Instruction], exceptions: &mut [CodeException]) {
    use crate::codegen::compiler::bytecode_emitter::method::inst_size;
    let mut byte_offsets = Vec::with_capacity(code.len() + 1);
    let mut offset = 0usize;
    for inst in code.iter() {
        byte_offsets.push(offset);
        offset += inst_size(inst);
    }
    byte_offsets.push(offset);


    for exc in exceptions.iter_mut() {
        if let Some(&start_off) = byte_offsets.get(exc.start_pc as usize) {
            exc.start_pc = start_off as u16;
        }
        if let Some(&end_off) = byte_offsets.get(exc.end_pc as usize) {
            exc.end_pc = end_off as u16;
        }
        if let Some(&handler_off) = byte_offsets.get(exc.handler_pc as usize) {
            exc.handler_pc = handler_off as u16;
        }
    }
}

pub struct BytecodeEmitter {
    pub(crate) cp: ConstantPool,
    pub(crate) methods: Vec<Method>,
    pub(crate) id_registry: HashMap<String, (u8, Type)>,
    pub(crate) local_slot: u8,
    pub(crate) class_path: Option<String>,
    pub(crate) super_class_path: Option<String>,
    pub(crate) all_classes: HashMap<String, ClassDef>,
    pub(crate) double_value_of: u16,
    pub(crate) double_double_value: u16,
    pub(crate) boolean_class: u16,
    pub(crate) boolean_value_of: u16,
    pub(crate) boolean_boolean_value: u16,
    pub(crate) array_list_class: u16,
    pub(crate) array_list_init: u16,
    pub(crate) array_list_add: u16,
    #[allow(dead_code)]
    pub(crate) linked_hash_map_class: u16,
    #[allow(dead_code)]
    pub(crate) linked_hash_map_init: u16,
    #[allow(dead_code)]
    pub(crate) linked_hash_map_put: u16,
    #[allow(dead_code)]
    pub(crate) linked_hash_map_get: u16,
    #[allow(dead_code)]
    pub(crate) runtime_exception_class: u16,
    #[allow(dead_code)]
    pub(crate) runtime_exception_init: u16,
}

impl BytecodeEmitter {
    pub fn new() -> Self {
        let mut cp = ConstantPool::default();
        let double_class = cp.add_class("java/lang/Double").unwrap();
        let double_value_of = cp.add_method_ref(double_class, "valueOf", "(D)Ljava/lang/Double;").unwrap();
        let double_double_value = cp.add_method_ref(double_class, "doubleValue", "()D").unwrap();
        
        let boolean_class = cp.add_class("java/lang/Boolean").unwrap();
        let boolean_value_of = cp.add_method_ref(boolean_class, "valueOf", "(Z)Ljava/lang/Boolean;").unwrap();
        let boolean_boolean_value = cp.add_method_ref(boolean_class, "booleanValue", "()Z").unwrap();
        
        let array_list_class = cp.add_class("java/util/ArrayList").unwrap();
        let array_list_init = cp.add_method_ref(array_list_class, "<init>", "()V").unwrap();
        let array_list_add = cp.add_method_ref(array_list_class, "add", "(Ljava/lang/Object;)Z").unwrap();
        
        let linked_hash_map_class = cp.add_class("java/util/LinkedHashMap").unwrap();
        let linked_hash_map_init = cp.add_method_ref(linked_hash_map_class, "<init>", "()V").unwrap();
        let linked_hash_map_put = cp.add_method_ref(linked_hash_map_class, "put", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;").unwrap();
        let linked_hash_map_get = cp.add_method_ref(linked_hash_map_class, "get", "(Ljava/lang/Object;)Ljava/lang/Object;").unwrap();

        let runtime_exception_class = cp.add_class("java/lang/RuntimeException").unwrap();
        let runtime_exception_init = cp.add_method_ref(runtime_exception_class, "<init>", "(Ljava/lang/String;)V").unwrap();

        Self {
            cp,
            methods: Vec::new(),
            id_registry: HashMap::new(),
            local_slot: 0,
            class_path: None,
            super_class_path: None,
            all_classes: HashMap::new(),
            double_value_of,
            double_double_value,
            boolean_class,
            boolean_value_of,
            boolean_boolean_value,
            array_list_class,
            array_list_init,
            array_list_add,
            linked_hash_map_class,
            linked_hash_map_init,
            linked_hash_map_put,
            linked_hash_map_get,
            runtime_exception_class,
            runtime_exception_init,
        }
    }

    pub fn emit_program(mut self, program: &[HirStmt], package_name: &str) -> Vec<(String, Vec<u8>)> {
        let mut classes = Vec::new();
        fn collect_classes(stmts: &[HirStmt], classes: &mut Vec<(String, ClassDef)>) {
            for stmt in stmts {
                match stmt {
                    HirStmt::ClassDecl(name, def) => {
                        classes.push((name.clone(), def.clone()));
                    }
                    HirStmt::If(_, cons, alt) => {
                        collect_classes(cons, classes);
                        collect_classes(alt, classes);
                    }
                    HirStmt::While(_, body) => {
                        collect_classes(body, classes);
                    }
                    HirStmt::DoWhile(body, _) => {
                        collect_classes(body, classes);
                    }
                    HirStmt::ForOf(_, _, _, body) => {
                        collect_classes(body, classes);
                    }
                    HirStmt::ForIn(_, _, body) => {
                        collect_classes(body, classes);
                    }
                    HirStmt::Switch(_, cases) => {
                        for case in cases {
                            collect_classes(&case.cons, classes);
                        }
                    }
                    HirStmt::TryCatch(try_b, _, catch_b, finally_b) => {
                        collect_classes(try_b, classes);
                        collect_classes(catch_b, classes);
                        collect_classes(finally_b, classes);
                    }
                    HirStmt::FnDecl(_, _, _, body, _) => {
                        collect_classes(body, classes);
                    }
                    HirStmt::Export(inner) => {
                        collect_classes(std::slice::from_ref(inner), classes);
                    }
                    _ => {}
                }
            }
        }
        collect_classes(program, &mut classes);
        let all_classes_map: HashMap<String, ClassDef> = classes.iter().cloned().collect();
        self.all_classes = all_classes_map.clone();

        let class_path = format!("{}/App", package_name.replace('.', "/"));
        self.class_path = Some(class_path.clone());
        let this_class = self.cp.add_class(&class_path).unwrap();
        let super_class = self.cp.add_class("java/lang/Object").unwrap();
        
        let init_name = self.cp.add_utf8("<init>").unwrap();
        let init_desc = self.cp.add_utf8("()V").unwrap();
        let super_init = self.cp.add_method_ref(super_class, "<init>".to_string(), "()V".to_string()).unwrap();
        let code_attr_name = self.cp.add_utf8("Code").unwrap();
        
        let init_code = vec![
            Instruction::Aload_0,
            Instruction::Invokespecial(super_init),
            Instruction::Return,
        ];
        
        self.methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC,
            name_index: init_name,
            descriptor_index: init_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr_name,
                max_stack: 1,
                max_locals: 1,
                code: init_code,
                exceptions: vec![],
                attributes: vec![],
            }],
        });
        
        for stmt in program {
            if let HirStmt::FnDecl(name, args, _ret_ty, body, modifiers) = stmt {
                let name_idx = self.cp.add_utf8(name).unwrap();
                let desc_idx = self.cp.add_utf8("([Ljava/lang/Object;)Ljava/lang/Object;").unwrap();
                
                self.id_registry.clear();
                self.local_slot = 0;
                
                let is_main = name == "main";
                if is_main {
                    let desc_idx_main = self.cp.add_utf8("([Ljava/lang/String;)V").unwrap();
                    self.local_slot = 1;
                    
                    let mut generator = crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen {
                        code: Vec::new(),
                        emitter: &mut self,
                        loop_stack: Vec::new(),
                        exceptions: Vec::new(),
                        is_async: false, // The entrypoint main method is always synchronous on the JVM
                        pending_label: None,
                    };
                    
                    generator.emit_stmts(body);
                    
                    let last_inst = generator.code.last();
                    if !matches!(last_inst, Some(Instruction::Areturn) | Some(Instruction::Return) | Some(Instruction::Ireturn) | Some(Instruction::Dreturn) | Some(Instruction::Athrow)) {
                        generator.code.push(Instruction::Return);
                    }
                    
                    let mut code = generator.code;
                    let mut exceptions = generator.exceptions;
                    let max_locals = 255;
                    let initial_locals = vec![ristretto_classfile::attributes::VerificationType::Object {
                        cpool_index: generator.emitter.cp.add_class("[Ljava/lang/Object;").unwrap_or(0),
                    }];
                    remove_dead_code(&mut code, &mut exceptions);
                    let smt = generate_stack_map_table(&mut generator.emitter.cp, &code, &exceptions, initial_locals);
                    resolve_jump_targets(&mut code, &mut exceptions);
                    
                    generator.emitter.methods.push(Method {
                        access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                        name_index: name_idx,
                        descriptor_index: desc_idx_main,
                        attributes: vec![Attribute::Code {
                            name_index: code_attr_name,
                            max_stack: 100,
                            max_locals,
                            code,
                            exceptions,
                            attributes: smt,
                        }],
                    });
                    continue;
                }
                
                let args_array_slot = 0;
                self.local_slot = 1;
                
                let mut generator = crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen {
                    code: Vec::new(),
                    emitter: &mut self,
                    loop_stack: Vec::new(),
                    exceptions: Vec::new(),
                    is_async: modifiers.is_async,
                    pending_label: None,
                };
                
                for (i, arg) in args.iter().enumerate() {
                    let slot = generator.emitter.local_slot;
                    if arg.ty == Type::Double {
                        generator.emitter.local_slot += 2;
                    } else {
                        generator.emitter.local_slot += 1;
                    }
                    generator.emitter.id_registry.insert(arg.name.clone(), (slot, arg.ty.clone()));
                    
                    if arg.is_rest {
                        let array_list_class = generator.emitter.cp.add_class("java/util/ArrayList").unwrap();
                        let array_list_init = generator.emitter.cp.add_method_ref(array_list_class, "<init>".to_string(), "()V".to_string()).unwrap();
                        let array_list_add = generator.emitter.cp.add_method_ref(array_list_class, "add".to_string(), "(Ljava/lang/Object;)Z".to_string()).unwrap();
                        
                        generator.code.push(Instruction::New(array_list_class));
                        generator.code.push(Instruction::Dup);
                        generator.code.push(Instruction::Invokespecial(array_list_init));
                        generator.code.push(Instruction::Astore(slot));
                        
                        let temp_idx_slot = generator.emitter.local_slot;
                        generator.emitter.local_slot += 1;
                        
                        generator.code.push(Instruction::Bipush(i as i8));
                        generator.code.push(Instruction::Istore(temp_idx_slot));
                        
                        let loop_start_pc = generator.code.len();
                        
                        generator.code.push(Instruction::Iload(temp_idx_slot));
                        generator.code.push(Instruction::Aload(args_array_slot));
                        generator.code.push(Instruction::Arraylength);
                        
                        let if_icmpge_idx = generator.code.len();
                        generator.code.push(Instruction::If_icmpge(0)); // Placeholder
                        
                        generator.code.push(Instruction::Aload(slot));
                        generator.code.push(Instruction::Aload(args_array_slot));
                        generator.code.push(Instruction::Iload(temp_idx_slot));
                        generator.code.push(Instruction::Aaload);
                        generator.code.push(Instruction::Invokevirtual(array_list_add));
                        generator.code.push(Instruction::Pop);
                        
                        generator.code.push(Instruction::Iinc(temp_idx_slot, 1));
                        
                        generator.code.push(Instruction::Goto(loop_start_pc as u16));
                        
                        let end_pc = generator.code.len();
                        generator.code[if_icmpge_idx] = Instruction::If_icmpge(end_pc as u16);
                    } else {
                        generator.code.push(Instruction::Aload(args_array_slot));
                        generator.code.push(Instruction::Bipush(i as i8));
                        generator.code.push(Instruction::Aaload);
                        
                        generator.unbox_if_needed(&arg.ty);
                        
                        match arg.ty {
                            Type::Double => generator.code.push(Instruction::Dstore(slot)),
                            Type::Int => generator.code.push(Instruction::Istore(slot)),
                            Type::Bool => generator.code.push(Instruction::Istore(slot)),
                            _ => generator.code.push(Instruction::Astore(slot)),
                        }
                    }
                }

                generator.emit_stmts(body);
                
                let last_inst = generator.code.last();
                if !matches!(last_inst, Some(Instruction::Areturn) | Some(Instruction::Return) | Some(Instruction::Ireturn) | Some(Instruction::Dreturn) | Some(Instruction::Athrow)) {
                    generator.code.push(Instruction::Aconst_null);
                    generator.code.push(Instruction::Areturn);
                }

                let mut code = generator.code;
                let mut exceptions = generator.exceptions;
                let max_locals = 255;
                let initial_locals = vec![ristretto_classfile::attributes::VerificationType::Object {
                    cpool_index: self.cp.add_class("[Ljava/lang/Object;").unwrap_or(0),
                }];
                remove_dead_code(&mut code, &mut exceptions);
                let smt = generate_stack_map_table(&mut self.cp, &code, &exceptions, initial_locals);
                resolve_jump_targets(&mut code, &mut exceptions);

                self.methods.push(Method {
                    access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                    name_index: name_idx,
                    descriptor_index: desc_idx,
                    attributes: vec![Attribute::Code {
                        name_index: code_attr_name,
                        max_stack: 100,
                        max_locals,
                        code,
                        exceptions,
                        attributes: smt,
                    }],
                });
            }
        }
        
        let mut clinit_code = Vec::new();
        for stmt in program {
            if let HirStmt::Import(bindings, module_name) = stmt {
                if bindings.is_empty() {
                    let class_class = self.cp.add_class("java/lang/Class").unwrap();
                    let for_name = self.cp.add_method_ref(class_class, "forName".to_string(), "(Ljava/lang/String;)Ljava/lang/Class;".to_string()).unwrap();
                    let name_idx = self.cp.add_string(module_name).unwrap();
                    clinit_code.push(Instruction::Ldc_w(name_idx));
                    clinit_code.push(Instruction::Invokestatic(for_name));
                    clinit_code.push(Instruction::Pop);
                }
            }
        }
        if !clinit_code.is_empty() {
            clinit_code.push(Instruction::Return);
            let clinit_name = self.cp.add_utf8("<clinit>").unwrap();
            let clinit_desc = self.cp.add_utf8("()V").unwrap();
            self.methods.push(Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: clinit_name,
                descriptor_index: clinit_desc,
                attributes: vec![Attribute::Code {
                    name_index: code_attr_name,
                    max_stack: 2,
                    max_locals: 0,
                    code: clinit_code,
                    exceptions: vec![],
                    attributes: vec![],
                }],
            });
        }
        let class_file = ClassFile {
            version: Version::Java21 { minor: 0 },
            constant_pool: self.cp,
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class,
            super_class,
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: self.methods,
            attributes: Vec::new(),
        };

        let mut buf = Vec::new();
        class_file.to_bytes(&mut buf).unwrap();
        
        let mut results = vec![(class_path, buf)];
        


        for (class_name, class_def) in &classes {
            let cp_path = format!("{}/{}", package_name.replace('.', "/"), class_name);
            let mut class_emitter = BytecodeEmitter::new();
            class_emitter.all_classes = all_classes_map.clone();
            let class_bytes = class_emitter.emit_class(&cp_path, class_def);
            results.push((cp_path, class_bytes));
        }
        
        results
    }

    pub fn emit_class(&mut self, class_path: &str, class_def: &ClassDef) -> Vec<u8> {
        self.class_path = Some(class_path.to_string());
        let this_class = self.cp.add_class(class_path).unwrap();
        
        let super_class_name = if let Some(extends) = &class_def.extends {
            if !extends.contains('/') && !extends.contains('.') {
                if let Some((package_name, _)) = class_path.rsplit_once('/') {
                    format!("{}/{}", package_name, extends)
                } else {
                    extends.clone()
                }
            } else {
                extends.replace('.', "/")
            }
        } else {
            "java/lang/Object".to_string()
        };
        let super_class = self.cp.add_class(&super_class_name).unwrap();
        self.super_class_path = Some(super_class_name.clone());
        
        let code_attr_name = self.cp.add_utf8("Code").unwrap();
        let mut fields = Vec::new();
        
        let mut members = class_def.members.clone();
        let has_ctor = members.iter().any(|m| matches!(m, ClassMember::Constructor(_, _)));
        if !has_ctor {
            members.push(ClassMember::Constructor(vec![], vec![]));
        }
        
        for member in &members {
            match member {
                ClassMember::Field(name, _ty, _init, mods) => {
                    let name_idx = self.cp.add_utf8(name).unwrap();
                    let desc_idx = self.cp.add_utf8("Ljava/lang/Object;").unwrap();
                    let access = if mods.is_static {
                        FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC
                    } else if mods.is_private {
                        FieldAccessFlags::PRIVATE
                    } else {
                        FieldAccessFlags::PUBLIC
                    };
                    fields.push(Field {
                        access_flags: access,
                        name_index: name_idx,
                        descriptor_index: desc_idx,
                        field_type: FieldType::Object("java/lang/Object".to_string()),
                        attributes: vec![],
                    });
                }
                ClassMember::Constructor(args, body) => {
                    let name_idx = self.cp.add_utf8("<init>").unwrap();
                    let desc_idx = self.cp.add_utf8("([Ljava/lang/Object;)V").unwrap();

                    self.id_registry.clear();
                    self.local_slot = 1; // 0 is `this`

                    let args_array_slot = 1;
                    self.local_slot = 2;

                    let mut generator = crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen {
                        code: Vec::new(),
                        emitter: self,
                        loop_stack: Vec::new(),
                        exceptions: Vec::new(),
                        is_async: false,
                        pending_label: None,
                    };

                    let mut has_super_call = false;
                    for s in body {
                        if matches!(s, HirStmt::Expr(HirExpr::SuperCall(..))) {
                            has_super_call = true;
                        }
                    }
                    if !has_super_call {
                        generator.code.push(Instruction::Aload_0);
                        if super_class_name == "java/lang/Object" {
                            let super_init = generator.emitter.cp.add_method_ref(super_class, "<init>".to_string(), "()V".to_string()).unwrap();
                            generator.code.push(Instruction::Invokespecial(super_init));
                        } else {
                            generator.code.push(Instruction::Bipush(0));
                            let obj_class = generator.emitter.cp.add_class("java/lang/Object").unwrap();
                            generator.code.push(Instruction::Anewarray(obj_class));
                            let super_init = generator.emitter.cp.add_method_ref(super_class, "<init>".to_string(), "([Ljava/lang/Object;)V".to_string()).unwrap();
                            generator.code.push(Instruction::Invokespecial(super_init));
                        }
                    }

                    // Initialize instance fields
                    let mut instance_inits = Vec::new();
                    for m in &members {
                        if let ClassMember::Field(fname, fty, Some(init_expr), fmods) = m {
                            if !fmods.is_static {
                                instance_inits.push((fname, fty, init_expr));
                            }
                        }
                    }
                    for (fname, _fty, init_expr) in &instance_inits {
                        generator.code.push(Instruction::Aload_0);
                        generator.emit_expr(init_expr);
                        let init_ty = generator.resolve_type(init_expr);
                        generator.box_if_needed(&init_ty);
                        let class_idx = generator.emitter.cp.add_class(&generator.emitter.class_path.clone().unwrap()).unwrap();
                        let field_idx = generator.emitter.cp.add_field_ref(class_idx, fname.to_string(), "Ljava/lang/Object;".to_string()).unwrap();
                        generator.code.push(Instruction::Putfield(field_idx));
                    }

                    // Unpack args
                    for (i, (arg_name, arg_ty)) in args.iter().enumerate() {
                        let slot = generator.emitter.local_slot;
                        if arg_ty == &Type::Double {
                            generator.emitter.local_slot += 2;
                        } else {
                            generator.emitter.local_slot += 1;
                        }
                        generator.emitter.id_registry.insert(arg_name.clone(), (slot, arg_ty.clone()));

                        generator.code.push(Instruction::Aload(args_array_slot));
                        generator.code.push(Instruction::Bipush(i as i8));
                        generator.code.push(Instruction::Aaload);

                        generator.unbox_if_needed(arg_ty);

                        match arg_ty {
                            Type::Double => generator.code.push(Instruction::Dstore(slot)),
                            Type::Int | Type::Bool => generator.code.push(Instruction::Istore(slot)),
                            _ => generator.code.push(Instruction::Astore(slot)),
                        }
                    }

                    generator.emit_stmts(body);

                    generator.code.push(Instruction::Return);

                    let mut code = generator.code;
                    let mut exceptions = generator.exceptions;
                    let max_locals = 255;
                    let initial_locals = vec![
                        ristretto_classfile::attributes::VerificationType::Object {
                            cpool_index: this_class,
                        },
                        ristretto_classfile::attributes::VerificationType::Object {
                            cpool_index: generator.emitter.cp.add_class("[Ljava/lang/Object;").unwrap_or(0),
                        }
                    ];
                    remove_dead_code(&mut code, &mut exceptions);
                    let smt = generate_stack_map_table(&mut generator.emitter.cp, &code, &exceptions, initial_locals);
                    resolve_jump_targets(&mut code, &mut exceptions);
                    generator.emitter.methods.push(Method {
                        access_flags: MethodAccessFlags::PUBLIC,
                        name_index: name_idx,
                        descriptor_index: desc_idx,
                        attributes: vec![Attribute::Code {
                            name_index: code_attr_name,
                            max_stack: 100,
                            max_locals,
                            code,
                            exceptions,
                            attributes: smt,
                        }],
                    });
                }
                ClassMember::Method(name, args, _ret_ty, body, mods) => {
                    let name_idx = self.cp.add_utf8(name).unwrap();
                    let is_supplier_bridge = name == "get" && args.is_empty();
                    let descriptor = if is_supplier_bridge {
                        "()Ljava/lang/Object;"
                    } else {
                        "([Ljava/lang/Object;)Ljava/lang/Object;"
                    };
                    let desc_idx = self.cp.add_utf8(descriptor).unwrap();

                    self.id_registry.clear();

                    let (args_array_slot, this_initial_local) = if is_supplier_bridge {
                        self.local_slot = 1; // 0 = this, no args array
                        (0u8, Some(ristretto_classfile::attributes::VerificationType::Object {
                            cpool_index: this_class,
                        }))
                    } else if mods.is_static {
                        self.local_slot = 1; // 0 = args array
                        (0u8, None)
                    } else {
                        self.local_slot = 2; // 0 = this, 1 = args array
                        (1u8, Some(ristretto_classfile::attributes::VerificationType::Object {
                            cpool_index: this_class,
                        }))
                    };

                    let mut generator = crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen {
                        code: Vec::new(),
                        emitter: self,
                        loop_stack: Vec::new(),
                        exceptions: Vec::new(),
                        is_async: mods.is_async,
                        pending_label: None,
                    };

                    // Unpack args
                    for (i, (arg_name, arg_ty)) in args.iter().enumerate() {
                        let slot = generator.emitter.local_slot;
                        if arg_ty == &Type::Double {
                            generator.emitter.local_slot += 2;
                        } else {
                            generator.emitter.local_slot += 1;
                        }
                        generator.emitter.id_registry.insert(arg_name.clone(), (slot, arg_ty.clone()));

                        generator.code.push(Instruction::Aload(args_array_slot));
                        generator.code.push(Instruction::Bipush(i as i8));
                        generator.code.push(Instruction::Aaload);

                        generator.unbox_if_needed(arg_ty);

                        match arg_ty {
                            Type::Double => generator.code.push(Instruction::Dstore(slot)),
                            Type::Int | Type::Bool => generator.code.push(Instruction::Istore(slot)),
                            _ => generator.code.push(Instruction::Astore(slot)),
                        }
                    }

                    generator.emit_stmts(body);

                    let last_inst = generator.code.last();
                    if !matches!(last_inst, Some(Instruction::Areturn) | Some(Instruction::Return) | Some(Instruction::Ireturn) | Some(Instruction::Dreturn) | Some(Instruction::Athrow)) {
                        generator.code.push(Instruction::Aconst_null);
                        generator.code.push(Instruction::Areturn);
                    }

                    let mut code = generator.code;
                    let mut exceptions = generator.exceptions;
                    let max_locals = 255;

                    let mut initial_locals = Vec::new();
                    if let Some(this_vt) = this_initial_local {
                        initial_locals.push(this_vt);
                    }
                    if !is_supplier_bridge {
                        initial_locals.push(ristretto_classfile::attributes::VerificationType::Object {
                            cpool_index: generator.emitter.cp.add_class("[Ljava/lang/Object;").unwrap_or(0),
                        });
                    }

                    remove_dead_code(&mut code, &mut exceptions);
                    let smt = generate_stack_map_table(&mut generator.emitter.cp, &code, &exceptions, initial_locals);
                    resolve_jump_targets(&mut code, &mut exceptions);

                    let access = if mods.is_static {
                        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC
                    } else if mods.is_private {
                        MethodAccessFlags::PRIVATE
                    } else {
                        MethodAccessFlags::PUBLIC
                    };

                    generator.emitter.methods.push(Method {
                        access_flags: access,
                        name_index: name_idx,
                        descriptor_index: desc_idx,
                        attributes: vec![Attribute::Code {
                            name_index: code_attr_name,
                            max_stack: 100,
                            max_locals,
                            code,
                            exceptions,
                            attributes: smt,
                        }],
                    });
                }

                // ── Getter: `get name()` → emitted as `get$name()` method ───────────
                ClassMember::Getter(prop_name, _ret_ty, body, is_static) => {
                    let method_name = format!("get${}", prop_name);
                    let name_idx = self.cp.add_utf8(&method_name).unwrap();
                    let desc_idx = self.cp.add_utf8("()Ljava/lang/Object;").unwrap();

                    self.local_slot = if *is_static { 0 } else { 1 }; // 0 = this for instance methods

                    let mut generator = crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen {
                        code: Vec::new(),
                        emitter: self,
                        loop_stack: Vec::new(),
                        exceptions: Vec::new(),
                        is_async: false,
                        pending_label: None,
                    };

                    generator.emit_stmts(body);

                    let last_inst = generator.code.last();
                    if !matches!(last_inst, Some(Instruction::Areturn) | Some(Instruction::Return) | Some(Instruction::Ireturn) | Some(Instruction::Dreturn) | Some(Instruction::Athrow)) {
                        generator.code.push(Instruction::Aconst_null);
                        generator.code.push(Instruction::Areturn);
                    }

                    let mut code = generator.code;
                    let mut exceptions = generator.exceptions;
                    
                    let initial_locals = if *is_static {
                        Vec::new()
                    } else {
                        vec![ristretto_classfile::attributes::VerificationType::Object {
                            cpool_index: this_class,
                        }]
                    };

                    remove_dead_code(&mut code, &mut exceptions);
                    let smt = generate_stack_map_table(&mut generator.emitter.cp, &code, &exceptions, initial_locals);
                    resolve_jump_targets(&mut code, &mut exceptions);
                    
                    let access = if *is_static {
                        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC
                    } else {
                        MethodAccessFlags::PUBLIC
                    };

                    generator.emitter.methods.push(Method {
                        access_flags: access,
                        name_index: name_idx,
                        descriptor_index: desc_idx,
                        attributes: vec![Attribute::Code {
                            name_index: code_attr_name,
                            max_stack: 100,
                            max_locals: 255,
                            code,
                            exceptions,
                            attributes: smt,
                        }],
                    });
                }

                // ── Setter: `set name(v)` → emitted as `set$name(Object)` method ────
                ClassMember::Setter(prop_name, param_name, _param_ty, body, is_static) => {
                    let method_name = format!("set${}", prop_name);
                    let name_idx = self.cp.add_utf8(&method_name).unwrap();
                    let desc_idx = self.cp.add_utf8("(Ljava/lang/Object;)V").unwrap();

                    self.id_registry.clear();
                    self.local_slot = if *is_static { 0 } else { 1 }; // 0 = this for instance methods
                    let param_slot = self.local_slot;
                    self.local_slot += 1;
                    self.id_registry.insert(param_name.clone(), (param_slot, Type::Any));

                    let mut generator = crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen {
                        code: Vec::new(),
                        emitter: self,
                        loop_stack: Vec::new(),
                        exceptions: Vec::new(),
                        is_async: false,
                        pending_label: None,
                    };

                    // Load the single parameter from local slot
                    generator.code.push(Instruction::Aload(param_slot));
                    generator.code.push(Instruction::Astore(param_slot)); // no-op but ensures slot is known

                    generator.emit_stmts(body);

                    generator.code.push(Instruction::Return);

                    let mut code = generator.code;
                    let mut exceptions = generator.exceptions;
                    
                    let initial_locals = if *is_static {
                        vec![
                            ristretto_classfile::attributes::VerificationType::Object {
                                cpool_index: generator.emitter.cp.add_class("java/lang/Object").unwrap_or(0),
                            },
                        ]
                    } else {
                        vec![
                            ristretto_classfile::attributes::VerificationType::Object {
                                cpool_index: this_class,
                            },
                            ristretto_classfile::attributes::VerificationType::Object {
                                cpool_index: generator.emitter.cp.add_class("java/lang/Object").unwrap_or(0),
                            },
                        ]
                    };

                    remove_dead_code(&mut code, &mut exceptions);
                    let smt = generate_stack_map_table(&mut generator.emitter.cp, &code, &exceptions, initial_locals);
                    resolve_jump_targets(&mut code, &mut exceptions);
                    
                    let access = if *is_static {
                        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC
                    } else {
                        MethodAccessFlags::PUBLIC
                    };

                    generator.emitter.methods.push(Method {
                        access_flags: access,
                        name_index: name_idx,
                        descriptor_index: desc_idx,
                        attributes: vec![Attribute::Code {
                            name_index: code_attr_name,
                            max_stack: 100,
                            max_locals: 255,
                            code,
                            exceptions,
                            attributes: smt,
                        }],
                    });
                }

                _ => {}
            }
        }

        // Generate static initializer method (<clinit>) if there are static fields with initializers or static blocks
        let mut static_inits = Vec::new();
        let mut has_static_blocks = false;
        for m in &members {
            if let ClassMember::Field(fname, fty, Some(init_expr), fmods) = m {
                if fmods.is_static {
                    static_inits.push((fname, fty, init_expr));
                }
            } else if let ClassMember::StaticInit(_) = m {
                has_static_blocks = true;
            }
        }

        if !static_inits.is_empty() || has_static_blocks {
            let clinit_name = self.cp.add_utf8("<clinit>").unwrap();
            let clinit_desc = self.cp.add_utf8("()V").unwrap();
            
            self.id_registry.clear();
            self.local_slot = 0;
            
            let mut generator = crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen {
                code: Vec::new(),
                emitter: self,
                loop_stack: Vec::new(),
                exceptions: Vec::new(),
                is_async: false,
                pending_label: None,
            };
            
            // 1. Initialize static fields
            for (fname, _fty, init_expr) in &static_inits {
                generator.emit_expr(init_expr);
                let init_ty = generator.resolve_type(init_expr);
                generator.box_if_needed(&init_ty);
                let class_idx = generator.emitter.cp.add_class(&generator.emitter.class_path.clone().unwrap()).unwrap();
                let field_idx = generator.emitter.cp.add_field_ref(class_idx, fname.to_string(), "Ljava/lang/Object;".to_string()).unwrap();
                generator.code.push(Instruction::Putstatic(field_idx));
            }
            
            // 2. Run static initializer blocks
            for m in &members {
                if let ClassMember::StaticInit(body) = m {
                    generator.emit_stmts(body);
                }
            }
            
            generator.code.push(Instruction::Return);
            
            let mut code = generator.code;
            let mut exceptions = generator.exceptions;
            remove_dead_code(&mut code, &mut exceptions);
            let smt = generate_stack_map_table(&mut generator.emitter.cp, &code, &exceptions, Vec::new());
            resolve_jump_targets(&mut code, &mut exceptions);
            
            generator.emitter.methods.push(Method {
                access_flags: MethodAccessFlags::STATIC,
                name_index: clinit_name,
                descriptor_index: clinit_desc,
                attributes: vec![Attribute::Code {
                    name_index: code_attr_name,
                    max_stack: 100,
                    max_locals: 255,
                    code,
                    exceptions,
                    attributes: smt,
                }],
            });
        }

        
        let mut access_flags = ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER;
        if class_def.is_abstract {
            access_flags |= ClassAccessFlags::ABSTRACT;
        }

        let mut interfaces = Vec::new();
        for interface_name in &class_def.implements {
            let idx = self.cp.add_class(&interface_name.replace('.', "/")).unwrap();
            interfaces.push(idx);
        }
        
        let class_file = ClassFile {
            version: Version::Java21 { minor: 0 },
            constant_pool: self.cp.clone(),
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods: self.methods.clone(),
            attributes: Vec::new(),
        };
        
        let mut buf = Vec::new();
        class_file.to_bytes(&mut buf).unwrap();
        buf
    }
}

pub fn remove_dead_code(code: &mut Vec<Instruction>, exceptions: &mut Vec<CodeException>) {
    if code.is_empty() { return; }
    let mut reachable = vec![false; code.len()];
    let mut worklist = vec![0];
    
    for exc in exceptions.iter() {
        worklist.push(exc.handler_pc as usize);
    }
    
    while let Some(i) = worklist.pop() {
        if i >= code.len() || reachable[i] { continue; }
        reachable[i] = true;
        
        match &code[i] {
            Instruction::Goto(t) => {
                worklist.push(*t as usize);
            }
            Instruction::Ifeq(t) | Instruction::Ifne(t) | Instruction::Iflt(t) | Instruction::Ifle(t) |
            Instruction::Ifgt(t) | Instruction::Ifge(t) | Instruction::Ifnull(t) | Instruction::Ifnonnull(t) |
            Instruction::If_icmpeq(t) | Instruction::If_icmpne(t) | Instruction::If_icmplt(t) | Instruction::If_icmple(t) |
            Instruction::If_icmpgt(t) | Instruction::If_icmpge(t) | Instruction::If_acmpeq(t) | Instruction::If_acmpne(t) => {
                worklist.push(i + 1);
                worklist.push(*t as usize);
            }
            Instruction::Ireturn | Instruction::Lreturn | Instruction::Freturn |
            Instruction::Dreturn | Instruction::Areturn | Instruction::Return |
            Instruction::Athrow => {
                // terminal
            }
            _ => {
                worklist.push(i + 1);
            }
        }
    }
    
    let mut new_code = Vec::new();
    let mut old_to_new = vec![0; code.len() + 1];
    let mut current_new = 0;
    
    for i in 0..code.len() {
        old_to_new[i] = current_new;
        if reachable[i] {
            new_code.push(code[i].clone());
            current_new += 1;
        }
    }
    old_to_new[code.len()] = current_new;
    
    // Rewrite jump targets
    for inst in new_code.iter_mut() {
        match inst {
            Instruction::Goto(t) | Instruction::Ifeq(t) | Instruction::Ifne(t) |
            Instruction::Iflt(t) | Instruction::Ifle(t) | Instruction::Ifgt(t) |
            Instruction::Ifge(t) | Instruction::Ifnull(t) | Instruction::Ifnonnull(t) |
            Instruction::If_icmpeq(t) | Instruction::If_icmpne(t) | Instruction::If_icmplt(t) |
            Instruction::If_icmple(t) | Instruction::If_icmpgt(t) | Instruction::If_icmpge(t) |
            Instruction::If_acmpeq(t) | Instruction::If_acmpne(t) => {
                if (*t as usize) < old_to_new.len() {
                    *t = old_to_new[*t as usize] as u16;
                }
            }
            _ => {}
        }
    }
    
    // Rewrite exceptions
    for exc in exceptions.iter_mut() {
        exc.start_pc = old_to_new[exc.start_pc as usize] as u16;
        exc.end_pc = old_to_new[exc.end_pc as usize] as u16;
        exc.handler_pc = old_to_new[exc.handler_pc as usize] as u16;
    }
    
    *code = new_code;
}
