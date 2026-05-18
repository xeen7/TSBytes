use ristretto_classfile::attributes::Instruction;
use crate::codegen::compiler::ir::*;
use crate::codegen::compiler::bytecode_emitter::method::MethodCodeGen;

impl<'a> MethodCodeGen<'a> {
    pub(crate) fn emit_array_ops(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::ArrayLen(obj) => {
                            self.emit_expr(obj);
                            let array_list_class = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                            self.code.push(Instruction::Checkcast(array_list_class));
                            let size_m = self.emitter.cp.add_method_ref(array_list_class, "size".to_string(), "()I".to_string()).unwrap();
                            self.code.push(Instruction::Invokevirtual(size_m));
                            // Convert int to double for JS number semantics
                            self.code.push(Instruction::I2d);
                        }
            HirExpr::ArrayMethod(obj, method_name, args, _ty) => {
                            let array_list_class = self.emitter.cp.add_class("java/util/ArrayList").unwrap();
                            match method_name.as_str() {
                                "push" => {
                                    // arr.push(x) → ArrayList.add(Object): boolean, return new size
                                    self.emit_expr(obj);
                                    for arg in args {
                                        self.code.push(Instruction::Dup);
                                        self.emit_expr(arg);
                                        self.box_if_needed(&arg.get_type());
                                        let add_m = self.emitter.cp.add_method_ref(array_list_class, "add".to_string(), "(Ljava/lang/Object;)Z".to_string()).unwrap();
                                        self.code.push(Instruction::Invokevirtual(add_m));
                                        self.code.push(Instruction::Pop); // discard boolean
                                    }
                                    // Return new length
                                    let size_m = self.emitter.cp.add_method_ref(array_list_class, "size".to_string(), "()I".to_string()).unwrap();
                                    self.code.push(Instruction::Invokevirtual(size_m));
                                    self.code.push(Instruction::I2d);
                                }
                                "pop" => {
                                    // arr.pop() → ArrayList.remove(size - 1)
                                    self.emit_expr(obj);
                                    self.code.push(Instruction::Dup);
                                    let size_m = self.emitter.cp.add_method_ref(array_list_class, "size".to_string(), "()I".to_string()).unwrap();
                                    self.code.push(Instruction::Invokevirtual(size_m));
                                    self.code.push(Instruction::Iconst_1);
                                    self.code.push(Instruction::Isub);
                                    let remove_m = self.emitter.cp.add_method_ref(array_list_class, "remove".to_string(), "(I)Ljava/lang/Object;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokevirtual(remove_m));
                                }
                                "shift" => {
                                    // arr.shift() → ArrayList.remove(0)
                                    self.emit_expr(obj);
                                    self.code.push(Instruction::Iconst_0);
                                    let remove_m = self.emitter.cp.add_method_ref(array_list_class, "remove".to_string(), "(I)Ljava/lang/Object;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokevirtual(remove_m));
                                }
                                "unshift" => {
                                    // arr.unshift(x) → ArrayList.add(0, Object); return new size
                                    self.emit_expr(obj);
                                    for arg in args {
                                        self.code.push(Instruction::Dup);
                                        self.code.push(Instruction::Iconst_0);
                                        self.emit_expr(arg);
                                        self.box_if_needed(&arg.get_type());
                                        let add_idx_m = self.emitter.cp.add_method_ref(array_list_class, "add".to_string(), "(ILjava/lang/Object;)V".to_string()).unwrap();
                                        self.code.push(Instruction::Invokevirtual(add_idx_m));
                                    }
                                    let size_m = self.emitter.cp.add_method_ref(array_list_class, "size".to_string(), "()I".to_string()).unwrap();
                                    self.code.push(Instruction::Invokevirtual(size_m));
                                    self.code.push(Instruction::I2d);
                                }
                                "indexOf" => {
                                    // arr.indexOf(x) → ArrayList.indexOf(Object): int → double
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.box_if_needed(&args[0].get_type());
                                    let indexof_m = self.emitter.cp.add_method_ref(array_list_class, "indexOf".to_string(), "(Ljava/lang/Object;)I".to_string()).unwrap();
                                    self.code.push(Instruction::Invokevirtual(indexof_m));
                                    self.code.push(Instruction::I2d);
                                }
                                "includes" => {
                                    // arr.includes(x) → ArrayList.contains(Object): boolean
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.box_if_needed(&args[0].get_type());
                                    let contains_m = self.emitter.cp.add_method_ref(array_list_class, "contains".to_string(), "(Ljava/lang/Object;)Z".to_string()).unwrap();
                                    self.code.push(Instruction::Invokevirtual(contains_m));
                                }
                                "join" => {
                                    // arr.join(sep?) → TsRuntime.arrayJoin(ArrayList, String): String
                                    self.emit_expr(obj);
                                    if args.is_empty() {
                                        let comma = self.emitter.cp.add_string(",").unwrap();
                                        self.code.push(Instruction::Ldc_w(comma));
                                    } else {
                                        self.emit_expr(&args[0]);
                                    }
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let join_m = self.emitter.cp.add_method_ref(runtime_class, "arrayJoin".to_string(), "(Ljava/util/ArrayList;Ljava/lang/String;)Ljava/lang/String;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(join_m));
                                }
                                "reverse" => {
                                    // Collections.reverse(list); return list
                                    self.emit_expr(obj);
                                    self.code.push(Instruction::Dup);
                                    let collections_class = self.emitter.cp.add_class("java/util/Collections").unwrap();
                                    let reverse_m = self.emitter.cp.add_method_ref(collections_class, "reverse".to_string(), "(Ljava/util/List;)V".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(reverse_m));
                                }
                                "slice" => {
                                    // TsRuntime.arraySlice(ArrayList, int fromIndex, int toIndex): ArrayList
                                    self.emit_expr(obj);
                                    if args.is_empty() {
                                        self.code.push(Instruction::Iconst_0);
                                        // Use Integer.MAX_VALUE as sentinel
                                        self.code.push(Instruction::Ldc_w(self.emitter.cp.add_integer(i32::MAX).unwrap()));
                                    } else if args.len() == 1 {
                                        self.emit_expr(&args[0]);
                                        self.unbox_to_int(&args[0].get_type());
                                        self.code.push(Instruction::Ldc_w(self.emitter.cp.add_integer(i32::MAX).unwrap()));
                                    } else {
                                        self.emit_expr(&args[0]);
                                        self.unbox_to_int(&args[0].get_type());
                                        self.emit_expr(&args[1]);
                                        self.unbox_to_int(&args[1].get_type());
                                    }
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let slice_m = self.emitter.cp.add_method_ref(runtime_class, "arraySlice".to_string(), "(Ljava/util/ArrayList;II)Ljava/util/ArrayList;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(slice_m));
                                }
                                "concat" => {
                                    // TsRuntime.arrayConcat(ArrayList, ArrayList): ArrayList
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let concat_m = self.emitter.cp.add_method_ref(runtime_class, "arrayConcat".to_string(), "(Ljava/util/ArrayList;Ljava/util/ArrayList;)Ljava/util/ArrayList;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(concat_m));
                                }
                                "splice" => {
                                    // TsRuntime.arraySplice(ArrayList, int start, int deleteCount): ArrayList
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.unbox_to_int(&args[0].get_type());
                                    if args.len() > 1 {
                                        self.emit_expr(&args[1]);
                                        self.unbox_to_int(&args[1].get_type());
                                    } else {
                                        self.code.push(Instruction::Ldc_w(self.emitter.cp.add_integer(i32::MAX).unwrap()));
                                    }
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let splice_m = self.emitter.cp.add_method_ref(runtime_class, "arraySplice".to_string(), "(Ljava/util/ArrayList;II)Ljava/util/ArrayList;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(splice_m));
                                }
                                "forEach" => {
                                    // TsRuntime.arrayForEach(ArrayList, Object callback): void
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.box_if_needed(&args[0].get_type());
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let m = self.emitter.cp.add_method_ref(runtime_class, "arrayForEach".to_string(), "(Ljava/util/ArrayList;Ljava/lang/Object;)V".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "map" => {
                                    // TsRuntime.arrayMap(ArrayList, Object callback): ArrayList
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.box_if_needed(&args[0].get_type());
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let m = self.emitter.cp.add_method_ref(runtime_class, "arrayMap".to_string(), "(Ljava/util/ArrayList;Ljava/lang/Object;)Ljava/util/ArrayList;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "filter" => {
                                    // TsRuntime.arrayFilter(ArrayList, Object callback): ArrayList
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.box_if_needed(&args[0].get_type());
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let m = self.emitter.cp.add_method_ref(runtime_class, "arrayFilter".to_string(), "(Ljava/util/ArrayList;Ljava/lang/Object;)Ljava/util/ArrayList;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "find" => {
                                    // TsRuntime.arrayFind(ArrayList, Object callback): Object
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.box_if_needed(&args[0].get_type());
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let m = self.emitter.cp.add_method_ref(runtime_class, "arrayFind".to_string(), "(Ljava/util/ArrayList;Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "reduce" => {
                                    // TsRuntime.arrayReduce(ArrayList, Object callback, Object initial): Object
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.box_if_needed(&args[0].get_type());
                                    if args.len() > 1 {
                                        self.emit_expr(&args[1]);
                                        self.box_if_needed(&args[1].get_type());
                                    } else {
                                        self.code.push(Instruction::Aconst_null);
                                    }
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let m = self.emitter.cp.add_method_ref(runtime_class, "arrayReduce".to_string(), "(Ljava/util/ArrayList;Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "some" => {
                                    // TsRuntime.arraySome(ArrayList, Object callback): boolean
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.box_if_needed(&args[0].get_type());
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let m = self.emitter.cp.add_method_ref(runtime_class, "arraySome".to_string(), "(Ljava/util/ArrayList;Ljava/lang/Object;)Z".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                "every" => {
                                    // TsRuntime.arrayEvery(ArrayList, Object callback): boolean
                                    self.emit_expr(obj);
                                    self.emit_expr(&args[0]);
                                    self.box_if_needed(&args[0].get_type());
                                    let runtime_class = self.emitter.cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
                                    let m = self.emitter.cp.add_method_ref(runtime_class, "arrayEvery".to_string(), "(Ljava/util/ArrayList;Ljava/lang/Object;)Z".to_string()).unwrap();
                                    self.code.push(Instruction::Invokestatic(m));
                                }
                                _ => {
                                    // Fallback: emit as generic method call
                                    self.emit_expr(obj);
                                    self.code.push(Instruction::Aconst_null);
                                }
                            }
                        }
            _ => unreachable!(),
        }
    }
}
