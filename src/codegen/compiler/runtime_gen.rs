/// TsObject Runtime Class Generator
///
/// Generates JVM bytecode for `com/tsdroid/runtime/TsObject`, a class
/// extending `java/util/LinkedHashMap` that adds:
/// - Prototype chain (`__proto__` field)
/// - `getProperty(String)` — walks prototype chain
/// - `setProperty(String, Object)` — sets own property
/// - `deleteProperty(String)` — deletes own property
/// - `hasProperty(String)` — checks own + prototype chain
/// - `hasOwnProperty(String)` — checks own only
/// - Static `Object.*` methods: keys, values, entries, assign, create, freeze
/// - `allKeys()` — for `for...in` iteration (own + prototype chain)

use ristretto_classfile::{
    ClassFile, ConstantPool, Method, Field, FieldType, Version,
    ClassAccessFlags, MethodAccessFlags, FieldAccessFlags,
    attributes::{Instruction, Attribute, StackFrame, VerificationType},
    BaseType,
};

const TS_OBJECT_PATH: &str = "com/tsdroid/runtime/TsObject";
const SUPER_PATH: &str = "java/util/LinkedHashMap";

pub fn generate_ts_object_class() -> (String, Vec<u8>) {
    // Constant pool for TsObject
    let mut cp = ConstantPool::default();

    let this_class = cp.add_class(TS_OBJECT_PATH).unwrap();
    let super_class = cp.add_class(SUPER_PATH).unwrap();
    let _object_class = cp.add_class("java/lang/Object").unwrap();
    let string_class = cp.add_class("java/lang/String").unwrap();
    let boolean_class = cp.add_class("java/lang/Boolean").unwrap();
    let arraylist_class = cp.add_class("java/util/ArrayList").unwrap();
    let set_class = cp.add_class("java/util/Set").unwrap();
    let hashset_class = cp.add_class("java/util/LinkedHashSet").unwrap();
    let iterator_class = cp.add_class("java/util/Iterator").unwrap();
    let _map_class = cp.add_class("java/util/Map").unwrap();
    let map_entry_class = cp.add_class("java/util/Map$Entry").unwrap();
    let system_class = cp.add_class("java/lang/System").unwrap();
    let identity_hash_code = cp.add_method_ref(system_class, "identityHashCode", "(Ljava/lang/Object;)I").unwrap();

    let code_attr = cp.add_utf8("Code").unwrap();

    // Super methods
    let super_init = cp.add_method_ref(super_class, "<init>", "()V").unwrap();
    let super_get = cp.add_method_ref(super_class, "get", "(Ljava/lang/Object;)Ljava/lang/Object;").unwrap();
    let super_put = cp.add_method_ref(super_class, "put", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;").unwrap();
    let super_remove = cp.add_method_ref(super_class, "remove", "(Ljava/lang/Object;)Ljava/lang/Object;").unwrap();
    let super_contains_key = cp.add_method_ref(super_class, "containsKey", "(Ljava/lang/Object;)Z").unwrap();
    let super_key_set = cp.add_method_ref(super_class, "keySet", "()Ljava/util/Set;").unwrap();
    let super_values_m = cp.add_method_ref(super_class, "values", "()Ljava/util/Collection;").unwrap();
    let super_entry_set = cp.add_method_ref(super_class, "entrySet", "()Ljava/util/Set;").unwrap();
    let super_put_all = cp.add_method_ref(super_class, "putAll", "(Ljava/util/Map;)V").unwrap();
    let linked_hash_map_get = super_get;
    let linked_hash_map_contains = super_contains_key;

    // ArrayList methods
    let al_init = cp.add_method_ref(arraylist_class, "<init>", "(Ljava/util/Collection;)V").unwrap();
    let al_init_empty = cp.add_method_ref(arraylist_class, "<init>", "()V").unwrap();
    let al_add = cp.add_method_ref(arraylist_class, "add", "(Ljava/lang/Object;)Z").unwrap();

    // Set/Iterator
    let set_iterator = cp.add_interface_method_ref(set_class, "iterator", "()Ljava/util/Iterator;").unwrap();
    let set_add_all = cp.add_interface_method_ref(set_class, "addAll", "(Ljava/util/Collection;)Z").unwrap();
    let iter_has_next = cp.add_interface_method_ref(iterator_class, "hasNext", "()Z").unwrap();
    let iter_next = cp.add_interface_method_ref(iterator_class, "next", "()Ljava/lang/Object;").unwrap();

    // LinkedHashSet
    let hashset_init = cp.add_method_ref(hashset_class, "<init>", "()V").unwrap();

    // Map.Entry
    let entry_get_key = cp.add_interface_method_ref(map_entry_class, "getKey", "()Ljava/lang/Object;").unwrap();
    let entry_get_value = cp.add_interface_method_ref(map_entry_class, "getValue", "()Ljava/lang/Object;").unwrap();

    // Boolean
    let _boolean_value_of = cp.add_method_ref(boolean_class, "valueOf", "(Z)Ljava/lang/Boolean;").unwrap();
    let boolean_boolean_value = cp.add_method_ref(boolean_class, "booleanValue", "()Z").unwrap();
    let double_class = cp.add_class("java/lang/Double").unwrap();
    let double_double_value = cp.add_method_ref(double_class, "doubleValue", "()D").unwrap();
    let double_is_nan = cp.add_method_ref(double_class, "isNaN", "(D)Z").unwrap();
    let string_is_empty = cp.add_method_ref(string_class, "isEmpty", "()Z").unwrap();

    // __proto__ field
    let proto_field = cp.add_field_ref(this_class, "__proto__", &format!("L{};", TS_OBJECT_PATH)).unwrap();
    let proto_field_name = cp.add_utf8("__proto__").unwrap();
    let proto_field_desc = cp.add_utf8(&format!("L{};", TS_OBJECT_PATH)).unwrap();

    // __getters__ / __setters__ fields (LinkedHashMap<String, Object>)
    let lhm_desc = format!("L{};", SUPER_PATH);
    let getters_field = cp.add_field_ref(this_class, "__getters__", &lhm_desc).unwrap();
    let getters_field_name = cp.add_utf8("__getters__").unwrap();
    let getters_field_desc = cp.add_utf8(&lhm_desc).unwrap();
    let setters_field = cp.add_field_ref(this_class, "__setters__", &lhm_desc).unwrap();
    let setters_field_name = cp.add_utf8("__setters__").unwrap();
    let setters_field_desc = cp.add_utf8(&lhm_desc).unwrap();

    // __frozen__ field (boolean)
    let frozen_field = cp.add_field_ref(this_class, "__frozen__", "Z").unwrap();
    let frozen_field_name = cp.add_utf8("__frozen__").unwrap();
    let frozen_field_desc = cp.add_utf8("Z").unwrap();

    // Method ref for invoking getter/setter lambdas: obj.invoke([Ljava/lang/Object;)Ljava/lang/Object;
    // We use Object class since we don't know the lambda class at compile time — will resolve at runtime via invokevirtual
    let object_class_ref = cp.add_class("java/lang/Object").unwrap();
    let _object_array_class = cp.add_class("[Ljava/lang/Object;").unwrap();
    let _lambda_invoke = cp.add_method_ref(object_class_ref, "invoke", "([Ljava/lang/Object;)Ljava/lang/Object;").unwrap();

    // Self method refs (for recursive getProperty calls on prototype)
    let self_get_property = cp.add_method_ref(this_class, "getProperty", "(Ljava/lang/String;)Ljava/lang/Object;").unwrap();
    let self_has_property = cp.add_method_ref(this_class, "hasProperty", "(Ljava/lang/String;)Z").unwrap();
    let self_all_keys = cp.add_method_ref(this_class, "allKeys", &format!("()Ljava/util/Set;")).unwrap();
    let self_init = cp.add_method_ref(this_class, "<init>", "()V").unwrap();

    let fields = vec![
        Field {
            access_flags: FieldAccessFlags::PUBLIC,
            name_index: proto_field_name,
            descriptor_index: proto_field_desc,
            field_type: FieldType::Object(TS_OBJECT_PATH.to_string()),
            attributes: vec![],
        },
        Field {
            access_flags: FieldAccessFlags::PUBLIC,
            name_index: getters_field_name,
            descriptor_index: getters_field_desc,
            field_type: FieldType::Object(SUPER_PATH.to_string()),
            attributes: vec![],
        },
        Field {
            access_flags: FieldAccessFlags::PUBLIC,
            name_index: setters_field_name,
            descriptor_index: setters_field_desc,
            field_type: FieldType::Object(SUPER_PATH.to_string()),
            attributes: vec![],
        },
        Field {
            access_flags: FieldAccessFlags::PUBLIC,
            name_index: frozen_field_name,
            descriptor_index: frozen_field_desc,
            field_type: FieldType::Base(BaseType::Boolean),
            attributes: vec![],
        },
    ];

    let mut methods = Vec::new();

    // === <init>()V ===
    // super(); this.__getters__ = new LinkedHashMap(); this.__setters__ = new LinkedHashMap();
    methods.push(make_method(&mut cp, code_attr, "<init>", "()V",
        MethodAccessFlags::PUBLIC,
        vec![
            Instruction::Aload_0,
            Instruction::Invokespecial(super_init),
            // this.__getters__ = new LinkedHashMap()
            Instruction::Aload_0,
            Instruction::New(super_class),
            Instruction::Dup,
            Instruction::Invokespecial(super_init),
            Instruction::Putfield(getters_field),
            // this.__setters__ = new LinkedHashMap()
            Instruction::Aload_0,
            Instruction::New(super_class),
            Instruction::Dup,
            Instruction::Invokespecial(super_init),
            Instruction::Putfield(setters_field),
            Instruction::Return,
        ], 3, 1));

    // === defineGetter(String, Object)V ===
    methods.push(make_method(&mut cp, code_attr, "defineGetter",
        "(Ljava/lang/String;Ljava/lang/Object;)V",
        MethodAccessFlags::PUBLIC,
        vec![
            Instruction::Aload_0,
            Instruction::Getfield(getters_field),
            Instruction::Aload(1), // key
            Instruction::Aload(2), // getter lambda
            Instruction::Invokevirtual(super_put),
            Instruction::Pop, // discard old value
            Instruction::Return,
        ], 4, 3));

    // === defineSetter(String, Object)V ===
    methods.push(make_method(&mut cp, code_attr, "defineSetter",
        "(Ljava/lang/String;Ljava/lang/Object;)V",
        MethodAccessFlags::PUBLIC,
        vec![
            Instruction::Aload_0,
            Instruction::Getfield(setters_field),
            Instruction::Aload(1), // key
            Instruction::Aload(2), // setter lambda
            Instruction::Invokevirtual(super_put),
            Instruction::Pop, // discard old value
            Instruction::Return,
        ], 4, 3));

    // === getProperty(String)Object ===
    // if (containsKey(key)) return get(key);
    // if (__proto__ != null) return __proto__.getProperty(key);
    // return null;
    //
    // Instruction layout with byte offsets:
    // 0: aload_0            (1)  -> byte 0
    // 1: aload 1            (2)  -> byte 1
    // 2: invokevirtual      (3)  -> byte 3
    // 3: ifeq -> 8          (3)  -> byte 6, target byte=16
    // 4: aload_0            (1)  -> byte 9
    // 5: aload 1            (2)  -> byte 10
    // 6: invokevirtual      (3)  -> byte 12
    // 7: areturn            (1)  -> byte 15
    // 8: aload_0            (1)  -> byte 16 <- target of ifeq
    // 9: getfield           (3)  -> byte 17
    // 10: ifnull -> 16      (3)  -> byte 20, target byte=33
    // 11: aload_0           (1)  -> byte 23
    // 12: getfield          (3)  -> byte 24
    // 13: aload 1           (2)  -> byte 27
    // 14: invokevirtual     (3)  -> byte 29
    // 15: areturn           (1)  -> byte 32
    // 16: aconst_null       (1)  -> byte 33 <- target of ifnull
    // 17: areturn           (1)  -> byte 34
    // === static invokeLambda(Object, Object[])Object ===
    // Dynamically invokes the `invoke` method on a Lambda$N instance
    {
        let class_class = cp.add_class("java/lang/Class").unwrap();
        let method_class = cp.add_class("java/lang/reflect/Method").unwrap();
        let get_class_m = cp.add_method_ref(object_class_ref, "getClass", "()Ljava/lang/Class;").unwrap();
        let get_method_m = cp.add_method_ref(class_class, "getMethod", "(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;").unwrap();
        let invoke_m = cp.add_method_ref(method_class, "invoke", "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;").unwrap();
        let _obj_array_class = cp.add_class("[Ljava/lang/Object;").unwrap();
        let _class_array_class = cp.add_class("java/lang/Class").unwrap();
        
        let invoke_str = cp.add_string("invoke").unwrap();
        let obj_array_cls = cp.add_class("[Ljava/lang/Object;").unwrap();
        
        methods.push(make_method_with_frames(&mut cp, code_attr, "invokeLambda", "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
            MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            vec![
                // Method m = lambda.getClass().getMethod("invoke", new Class[]{ Object[].class })
                Instruction::Aload_0,
                Instruction::Invokevirtual(get_class_m),
                Instruction::Ldc_w(invoke_str),
                Instruction::Iconst_1,
                Instruction::Anewarray(class_class),
                Instruction::Dup,
                Instruction::Iconst_0,
                // load Object[].class (using ldc on array type)
                Instruction::Ldc_w(obj_array_cls),
                Instruction::Aastore,
                Instruction::Invokevirtual(get_method_m),
                // m.invoke(lambda, new Object[]{ args })
                Instruction::Aload_0,
                Instruction::Iconst_1,
                Instruction::Anewarray(object_class_ref),
                Instruction::Dup,
                Instruction::Iconst_0,
                Instruction::Aload(1),
                Instruction::Aastore,
                Instruction::Invokevirtual(invoke_m),
                Instruction::Areturn,
            ], 6, 2, vec![]));
    }

    {
        let invoke_lambda_m = cp.add_method_ref(this_class, "invokeLambda", "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;").unwrap();
        let _ts_obj_vtype = VerificationType::Object { cpool_index: this_class };
        let _string_vtype = VerificationType::Object { cpool_index: cp.add_class("java/lang/String").unwrap() };
        methods.push(make_method_with_frames(&mut cp, code_attr, "getProperty", "(Ljava/lang/String;)Ljava/lang/Object;",
            MethodAccessFlags::PUBLIC,
            vec![
                // __getters__.containsKey(key)
                Instruction::Aload_0,
                Instruction::Getfield(getters_field),
                Instruction::Aload(1),
                Instruction::Invokevirtual(linked_hash_map_contains),
                Instruction::Ifeq(17), // jump to 17
                
                // return invokeLambda(__getters__.get(key), new Object[]{ this })
                Instruction::Aload_0,
                Instruction::Getfield(getters_field),
                Instruction::Aload(1),
                Instruction::Invokevirtual(linked_hash_map_get),
                Instruction::Iconst_1,
                Instruction::Anewarray(object_class_ref),
                Instruction::Dup,
                Instruction::Iconst_0,
                Instruction::Aload_0,
                Instruction::Aastore,
                Instruction::Invokestatic(invoke_lambda_m),
                Instruction::Areturn,
                
                // if (super.containsKey(key))
                Instruction::Aload_0, // index 17
                Instruction::Aload(1),
                Instruction::Invokevirtual(super_contains_key),
                Instruction::Ifeq(25), // jump to 25
                
                // return super.get(key)
                Instruction::Aload_0,
                Instruction::Aload(1),
                Instruction::Invokevirtual(super_get),
                Instruction::Areturn,
                
                // if (__proto__ != null)
                Instruction::Aload_0, // index 25
                Instruction::Getfield(proto_field),
                Instruction::Ifnull(33), // jump to 33
                
                // return __proto__.getProperty(key)
                Instruction::Aload_0,
                Instruction::Getfield(proto_field),
                Instruction::Aload(1),
                Instruction::Invokevirtual(self_get_property),
                Instruction::Areturn,
                
                // return null
                Instruction::Aconst_null, // index 33
                Instruction::Areturn,
            ], 5, 2,
            vec![
                StackFrame::SameFrame { frame_type: 33 },
                StackFrame::SameFrame { frame_type: 15 },
                StackFrame::SameFrame { frame_type: 16 },
            ]));
    }

    // === setProperty(String, Object)V ===
    // if (this.__frozen__) return;
    // if (this.__setters__.containsKey(key)) { invokeLambda(__setters__.get(key), new Object[]{this, value}); return; }
    // super.put(key, value); pop; return;
    {
        let invoke_lambda_m = cp.add_method_ref(this_class, "invokeLambda", "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;").unwrap();
        let _ts_obj_vtype = VerificationType::Object { cpool_index: this_class };
        let _string_vtype = VerificationType::Object { cpool_index: cp.add_class("java/lang/String").unwrap() };
        let _object_vtype = VerificationType::Object { cpool_index: cp.add_class("java/lang/Object").unwrap() };
        methods.push(make_method_with_frames(&mut cp, code_attr, "setProperty", "(Ljava/lang/String;Ljava/lang/Object;)V",
            MethodAccessFlags::PUBLIC,
            vec![
                // if (this.__frozen__) return;
                Instruction::Aload_0,
                Instruction::Getfield(frozen_field),
                Instruction::Ifeq(4), // jump to 4
                Instruction::Return,
                
                // if (__setters__.containsKey(key))
                Instruction::Aload_0, // index 4
                Instruction::Getfield(setters_field),
                Instruction::Aload(1),
                Instruction::Invokevirtual(linked_hash_map_contains),
                Instruction::Ifeq(26), // jump to 26
                
                // invokeLambda(__setters__.get(key), new Object[]{ this, value })
                Instruction::Aload_0,
                Instruction::Getfield(setters_field),
                Instruction::Aload(1),
                Instruction::Invokevirtual(linked_hash_map_get),
                Instruction::Iconst_2,
                Instruction::Anewarray(object_class_ref),
                Instruction::Dup,
                Instruction::Iconst_0,
                Instruction::Aload_0,
                Instruction::Aastore,
                Instruction::Dup,
                Instruction::Iconst_1,
                Instruction::Aload(2),
                Instruction::Aastore,
                Instruction::Invokestatic(invoke_lambda_m),
                Instruction::Pop,
                Instruction::Return,
                
                // super.put(key, value)
                Instruction::Aload_0, // index 26
                Instruction::Aload(1),
                Instruction::Aload(2),
                Instruction::Invokevirtual(super_put),
                Instruction::Pop,
                Instruction::Return,
            ], 6, 3,
            vec![
                StackFrame::SameFrame { frame_type: 8 },
                StackFrame::SameFrame { frame_type: 38 },
            ],
        ));
    }

    // === deleteProperty(String)Z ===
    // 0: aload_0           (1) byte 0
    // 1: aload 1           (2) byte 1
    // 2: invokevirtual     (3) byte 3
    // 3: ifeq -> 10        (3) byte 6, target byte=17
    // 4: aload_0           (1) byte 9
    // 5: aload 1           (2) byte 10
    // 6: invokevirtual     (3) byte 12
    // 7: pop               (1) byte 15
    // 8: iconst_1          (1) byte 16
    // 9: ireturn           (1) byte 17
    // 10: iconst_0         (1) byte 18 <- target
    // 11: ireturn          (1) byte 19
    methods.push(make_method_with_frames(&mut cp, code_attr, "deleteProperty", "(Ljava/lang/String;)Z",
        MethodAccessFlags::PUBLIC,
        vec![
            Instruction::Aload_0,
            Instruction::Aload(1),
            Instruction::Invokevirtual(super_contains_key),
            Instruction::Ifeq(10),
            Instruction::Aload_0,
            Instruction::Aload(1),
            Instruction::Invokevirtual(super_remove),
            Instruction::Pop,
            Instruction::Iconst_1,
            Instruction::Ireturn,
            Instruction::Iconst_0,
            Instruction::Ireturn,
        ], 2, 2,
        // Frame at byte 18 (after ifeq target). delta from method start = 18
        // But wait, the target of ifeq(10) → instruction 10 → byte 18
        // frame_type for SameFrame = offset_delta, first frame offset = byte offset
        vec![StackFrame::SameFrame { frame_type: 18 }],
    ));

    // === hasOwnProperty(String)Z ===
    methods.push(make_method(&mut cp, code_attr, "hasOwnProperty", "(Ljava/lang/String;)Z",
        MethodAccessFlags::PUBLIC,
        vec![
            Instruction::Aload_0,
            Instruction::Aload(1),
            Instruction::Invokevirtual(super_contains_key),
            Instruction::Ireturn,
        ], 2, 2));

    // === hasProperty(String)Z ===
    // 0: aload_0           (1) byte 0
    // 1: aload 1           (2) byte 1
    // 2: invokevirtual     (3) byte 3
    // 3: ifeq -> 6         (3) byte 6, target byte=11
    // 4: iconst_1          (1) byte 9
    // 5: ireturn           (1) byte 10
    // 6: aload_0           (1) byte 11 <- target of ifeq
    // 7: getfield          (3) byte 12
    // 8: ifnull -> 14      (3) byte 15, target byte=28
    // 9: aload_0           (1) byte 18
    // 10: getfield         (3) byte 19
    // 11: aload 1          (2) byte 22
    // 12: invokevirtual    (3) byte 24
    // 13: ireturn          (1) byte 27
    // 14: iconst_0         (1) byte 28 <- target of ifnull
    // 15: ireturn          (1) byte 29
    methods.push(make_method_with_frames(&mut cp, code_attr, "hasProperty", "(Ljava/lang/String;)Z",
        MethodAccessFlags::PUBLIC,
        vec![
            Instruction::Aload_0,
            Instruction::Aload(1),
            Instruction::Invokevirtual(super_contains_key),
            Instruction::Ifeq(6),
            Instruction::Iconst_1,
            Instruction::Ireturn,
            Instruction::Aload_0,
            Instruction::Getfield(proto_field),
            Instruction::Ifnull(14),
            Instruction::Aload_0,
            Instruction::Getfield(proto_field),
            Instruction::Aload(1),
            Instruction::Invokevirtual(self_has_property),
            Instruction::Ireturn,
            Instruction::Iconst_0,
            Instruction::Ireturn,
        ], 3, 2,
        vec![
            // Frame at byte 11 (instruction 6)
            StackFrame::SameFrame { frame_type: 11 },
            // Frame at byte 28 (instruction 14). delta = 28 - 11 - 1 = 16
            StackFrame::SameFrame { frame_type: 16 },
        ],
    ));

    // === static keys(TsObject)Object ===
    // return new ArrayList(obj.keySet());
    methods.push(make_method(&mut cp, code_attr, "keys",
        &format!("(L{};)Ljava/lang/Object;", TS_OBJECT_PATH),
        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        vec![
            Instruction::New(arraylist_class),
            Instruction::Dup,
            Instruction::Aload_0,
            Instruction::Invokevirtual(super_key_set),
            Instruction::Invokespecial(al_init),
            Instruction::Areturn,
        ], 4, 1));

    // === static values(TsObject)Object ===
    methods.push(make_method(&mut cp, code_attr, "values",
        &format!("(L{};)Ljava/lang/Object;", TS_OBJECT_PATH),
        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        vec![
            Instruction::New(arraylist_class),
            Instruction::Dup,
            Instruction::Aload_0,
            Instruction::Invokevirtual(super_values_m),
            Instruction::Invokespecial(al_init),
            Instruction::Areturn,
        ], 4, 1));

    // === static entries(TsObject)Object ===
    // Build ArrayList of [key, value] pairs
    methods.push(make_method_with_frames(&mut cp, code_attr, "entries",
        &format!("(L{};)Ljava/lang/Object;", TS_OBJECT_PATH),
        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        vec![
            // result = new ArrayList()
            Instruction::New(arraylist_class),
            Instruction::Dup,
            Instruction::Invokespecial(al_init_empty),
            Instruction::Astore(1),
            // iter = obj.entrySet().iterator()
            Instruction::Aload_0,
            Instruction::Invokevirtual(super_entry_set),
            Instruction::Invokeinterface(set_iterator, 1),
            Instruction::Astore(2),
            // while loop start
            Instruction::Aload(2),
            Instruction::Invokeinterface(iter_has_next, 1),
            Instruction::Ifeq(34), // jump to return
            // entry = iter.next()
            Instruction::Aload(2),
            Instruction::Invokeinterface(iter_next, 1),
            Instruction::Checkcast(map_entry_class),
            Instruction::Astore(3),
            // pair = new ArrayList()
            Instruction::New(arraylist_class),
            Instruction::Dup,
            Instruction::Invokespecial(al_init_empty),
            Instruction::Astore(4),
            // pair.add(entry.getKey())
            Instruction::Aload(4),
            Instruction::Aload(3),
            Instruction::Invokeinterface(entry_get_key, 1),
            Instruction::Invokevirtual(al_add),
            Instruction::Pop,
            // pair.add(entry.getValue())
            Instruction::Aload(4),
            Instruction::Aload(3),
            Instruction::Invokeinterface(entry_get_value, 1),
            Instruction::Invokevirtual(al_add),
            Instruction::Pop,
            // result.add(pair)
            Instruction::Aload(1),
            Instruction::Aload(4),
            Instruction::Invokevirtual(al_add),
            Instruction::Pop,
            // goto loop start
            Instruction::Goto(8),
            // return result
            Instruction::Aload(1),
            Instruction::Areturn,
        ], 4, 5,
        vec![
            StackFrame::FullFrame {
                frame_type: 255,
                offset_delta: 20,
                locals: vec![
                    ristretto_classfile::attributes::VerificationType::Object { cpool_index: this_class },
                    ristretto_classfile::attributes::VerificationType::Object { cpool_index: arraylist_class },
                    ristretto_classfile::attributes::VerificationType::Object { cpool_index: iterator_class },
                ],
                stack: vec![],
            },
            StackFrame::FullFrame {
                frame_type: 255,
                offset_delta: 67, // 88 - 20 - 1
                locals: vec![
                    ristretto_classfile::attributes::VerificationType::Object { cpool_index: this_class },
                    ristretto_classfile::attributes::VerificationType::Object { cpool_index: arraylist_class },
                    ristretto_classfile::attributes::VerificationType::Object { cpool_index: iterator_class },
                ],
                stack: vec![],
            },
        ]));

    // === static assign(TsObject, TsObject)TsObject ===
    methods.push(make_method(&mut cp, code_attr, "assign",
        &format!("(L{};L{};)L{};", TS_OBJECT_PATH, TS_OBJECT_PATH, TS_OBJECT_PATH),
        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        vec![
            Instruction::Aload_0, // target
            Instruction::Aload(1), // source
            Instruction::Invokevirtual(super_put_all),
            Instruction::Aload_0,
            Instruction::Areturn,
        ], 2, 2));

    // === static create(TsObject)TsObject ===
    methods.push(make_method(&mut cp, code_attr, "create",
        &format!("(L{};)L{};", TS_OBJECT_PATH, TS_OBJECT_PATH),
        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        vec![
            Instruction::New(this_class),
            Instruction::Dup,
            Instruction::Invokespecial(self_init),
            Instruction::Dup,
            Instruction::Aload_0, // proto arg
            Instruction::Putfield(proto_field),
            Instruction::Areturn,
        ], 3, 1));

    // === static freeze(TsObject)TsObject ===
    // obj.__frozen__ = true; return obj;
    methods.push(make_method(&mut cp, code_attr, "freeze",
        &format!("(L{};)L{};", TS_OBJECT_PATH, TS_OBJECT_PATH),
        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        vec![
            Instruction::Aload_0,
            Instruction::Iconst_1, // true
            Instruction::Putfield(frozen_field),
            Instruction::Aload_0,
            Instruction::Areturn,
        ], 2, 1));

    // === static getPrototypeOf(TsObject)TsObject ===
    methods.push(make_method(&mut cp, code_attr, "getPrototypeOf",
        &format!("(L{};)L{};", TS_OBJECT_PATH, TS_OBJECT_PATH),
        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        vec![
            Instruction::Aload_0,
            Instruction::Getfield(proto_field),
            Instruction::Areturn,
        ], 1, 1));

    // === static setPrototypeOf(TsObject, TsObject)V ===
    methods.push(make_method(&mut cp, code_attr, "setPrototypeOf",
        &format!("(L{};L{};)V", TS_OBJECT_PATH, TS_OBJECT_PATH),
        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        vec![
            Instruction::Aload_0,
            Instruction::Aload(1),
            Instruction::Putfield(proto_field),
            Instruction::Return,
        ], 2, 2));

    // === static defineProperty(TsObject, String, Object)TsObject ===
    {
        let has_property_m = cp.add_method_ref(this_class, "hasProperty", "(Ljava/lang/String;)Z").unwrap();
        let get_property_m = cp.add_method_ref(this_class, "getProperty", "(Ljava/lang/String;)Ljava/lang/Object;").unwrap();
        let define_getter_m = cp.add_method_ref(this_class, "defineGetter", "(Ljava/lang/String;Ljava/lang/Object;)V").unwrap();
        let define_setter_m = cp.add_method_ref(this_class, "defineSetter", "(Ljava/lang/String;Ljava/lang/Object;)V").unwrap();
        let set_property_m = cp.add_method_ref(this_class, "setProperty", "(Ljava/lang/String;Ljava/lang/Object;)V").unwrap();
        
        let get_str = cp.add_string("get").unwrap();
        let set_str = cp.add_string("set").unwrap();
        let val_str = cp.add_string("value").unwrap();
        let ts_obj_vtype = VerificationType::Object { cpool_index: this_class };
        
        let define_property_code = vec![
            // if (!(descriptor instanceof TsObject)) return obj;
            Instruction::Aload(2), // 0
            Instruction::Instanceof(this_class), // 1
            Instruction::Ifne(5), // 2 -> jump to 5
            Instruction::Aload_0, // 3
            Instruction::Areturn, // 4
            
            // TsObject desc = (TsObject) descriptor;
            Instruction::Aload(2), // 5 (byte 9)
            Instruction::Checkcast(this_class), // 6
            Instruction::Astore(3), // 7
            
            // if (desc.hasProperty("get"))
            Instruction::Aload(3), // 8
            Instruction::Ldc_w(get_str), // 9
            Instruction::Invokevirtual(has_property_m), // 10
            Instruction::Ifeq(18), // 11 -> jump to 18
            
            // obj.defineGetter(prop, desc.getProperty("get"));
            Instruction::Aload_0, // 12
            Instruction::Aload(1), // 13
            Instruction::Aload(3), // 14
            Instruction::Ldc_w(get_str), // 15
            Instruction::Invokevirtual(get_property_m), // 16
            Instruction::Invokevirtual(define_getter_m), // 17
            
            // next1: if (desc.hasProperty("set"))
            Instruction::Aload(3), // 18 (byte 36)
            Instruction::Ldc_w(set_str), // 19
            Instruction::Invokevirtual(has_property_m), // 20
            Instruction::Ifeq(28), // 21 -> jump to 28
            
            // obj.defineSetter(prop, desc.getProperty("set"));
            Instruction::Aload_0, // 22
            Instruction::Aload(1), // 23
            Instruction::Aload(3), // 24
            Instruction::Ldc_w(set_str), // 25
            Instruction::Invokevirtual(get_property_m), // 26
            Instruction::Invokevirtual(define_setter_m), // 27
            
            // next2: if (desc.hasProperty("value"))
            Instruction::Aload(3), // 28 (byte 58)
            Instruction::Ldc_w(val_str), // 29
            Instruction::Invokevirtual(has_property_m), // 30
            Instruction::Ifeq(38), // 31 -> jump to 38
            
            // obj.setProperty(prop, desc.getProperty("value"));
            Instruction::Aload_0, // 32
            Instruction::Aload(1), // 33
            Instruction::Aload(3), // 34
            Instruction::Ldc_w(val_str), // 35
            Instruction::Invokevirtual(get_property_m), // 36
            Instruction::Invokevirtual(set_property_m), // 37
            
            // end: return obj;
            Instruction::Aload_0, // 38 (byte 80)
            Instruction::Areturn, // 39
        ];

        let mut define_property_byte_offsets = vec![];
        let mut curr_offset = 0;
        for inst in &define_property_code {
            define_property_byte_offsets.push(curr_offset);
            curr_offset += crate::codegen::compiler::bytecode_emitter::method::inst_size(inst);
        }

        let dp_target1 = define_property_byte_offsets[5];
        let dp_target_next1 = define_property_byte_offsets[18];
        let dp_target_next2 = define_property_byte_offsets[28];
        let dp_target_end = define_property_byte_offsets[38];

        methods.push(make_method_with_frames(&mut cp, code_attr, "defineProperty", 
            &format!("(L{};Ljava/lang/String;Ljava/lang/Object;)L{};", TS_OBJECT_PATH, TS_OBJECT_PATH),
            MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            define_property_code, 4, 4,
            vec![
                StackFrame::SameFrame { frame_type: dp_target1 as u8 },
                StackFrame::AppendFrame { frame_type: 252, offset_delta: (dp_target_next1 - dp_target1 - 1) as u16, locals: vec![ts_obj_vtype.clone()] },
                StackFrame::SameFrame { frame_type: (dp_target_next2 - dp_target_next1 - 1) as u8 },
                StackFrame::SameFrame { frame_type: (dp_target_end - dp_target_next2 - 1) as u8 },
            ]));
    }

    // === static add(Object, Object)Object ===
    {
        let double_class = cp.add_class("java/lang/Double").unwrap();
        let double_value = cp.add_method_ref(double_class, "doubleValue".to_string(), "()D".to_string()).unwrap();
        let double_value_of = cp.add_method_ref(double_class, "valueOf".to_string(), "(D)Ljava/lang/Double;".to_string()).unwrap();
        let string_class = cp.add_class("java/lang/String").unwrap();
        let string_concat = cp.add_method_ref(string_class, "concat".to_string(), "(Ljava/lang/String;)Ljava/lang/String;".to_string()).unwrap();
        let runtime_class = cp.add_class("com/tsdroid/runtime/TsRuntime").unwrap();
        let ts_to_string = cp.add_method_ref(runtime_class, "tsToString".to_string(), "(Ljava/lang/Object;)Ljava/lang/String;".to_string()).unwrap();

        methods.push(make_method_with_frames(&mut cp, code_attr, "add",
            "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
            MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            vec![
                Instruction::Aload_0,
                Instruction::Instanceof(double_class),
                Instruction::Ifeq(15),
                Instruction::Aload_1,
                Instruction::Instanceof(double_class),
                Instruction::Ifeq(15),
                Instruction::Aload_0,
                Instruction::Checkcast(double_class),
                Instruction::Invokevirtual(double_value),
                Instruction::Aload_1,
                Instruction::Checkcast(double_class),
                Instruction::Invokevirtual(double_value),
                Instruction::Dadd,
                Instruction::Invokestatic(double_value_of),
                Instruction::Areturn,
                Instruction::Aload_0, // index 15
                Instruction::Invokestatic(ts_to_string),
                Instruction::Aload_1,
                Instruction::Invokestatic(ts_to_string),
                Instruction::Invokevirtual(string_concat),
                Instruction::Areturn,
            ], 4, 2,
            vec![
                StackFrame::SameFrame { frame_type: 33 }
            ]
        ));
    }

    // === static hasOwn(TsObject, String)Z ===
    methods.push(make_method(&mut cp, code_attr, "hasOwn",
        &format!("(L{};Ljava/lang/String;)Z", TS_OBJECT_PATH),
        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        vec![
            Instruction::Aload_0,
            Instruction::Aload(1),
            Instruction::Invokevirtual(super_contains_key),
            Instruction::Ireturn,
        ], 2, 2));

    // === allKeys()Set — own + prototype chain keys for for-in ===
    // 0: new               (3) byte 0
    // 1: dup               (1) byte 3
    // 2: invokespecial     (3) byte 4
    // 3: astore 1          (2) byte 7
    // 4: aload 1           (2) byte 9
    // 5: aload_0           (1) byte 11
    // 6: invokevirtual     (3) byte 12
    // 7: invokeinterface   (5) byte 15
    // 8: pop               (1) byte 20
    // 9: aload_0           (1) byte 21
    // 10: getfield         (3) byte 22
    // 11: ifnull -> 18     (3) byte 25, target byte=40
    // 12: aload 1          (2) byte 28
    // 13: aload_0          (1) byte 30
    // 14: getfield         (3) byte 31
    // 15: invokevirtual    (3) byte 34
    // 16: invokeinterface  (5) byte 37
    // 17: pop              (1) byte 42
    // 18: aload 1          (2) byte 43 <- target OR fallthrough
    // 19: areturn          (1) byte 45
    {
        let set_vtype = VerificationType::Object { cpool_index: hashset_class };
        methods.push(make_method_with_frames(&mut cp, code_attr, "allKeys", "()Ljava/util/Set;",
            MethodAccessFlags::PUBLIC,
            vec![
                Instruction::New(hashset_class),
                Instruction::Dup,
                Instruction::Invokespecial(hashset_init),
                Instruction::Astore(1),
                Instruction::Aload(1),
                Instruction::Aload_0,
                Instruction::Invokevirtual(super_key_set),
                Instruction::Invokeinterface(set_add_all, 2),
                Instruction::Pop,
                Instruction::Aload_0,
                Instruction::Getfield(proto_field),
                Instruction::Ifnull(18),
                Instruction::Aload(1),
                Instruction::Aload_0,
                Instruction::Getfield(proto_field),
                Instruction::Invokevirtual(self_all_keys),
                Instruction::Invokeinterface(set_add_all, 2),
                Instruction::Pop,
                Instruction::Aload(1),
                Instruction::Areturn,
            ], 4, 2,
            // Frame at byte offset of instruction 18.
            // Instruction 18 = aload 1 after the ifnull jump. Byte offset = ?
            // Let me recalculate: invokeinterface is 5 bytes (opcode + 2 index + count + 0)
            // Actually ristretto invokeinterface(ref, count) encodes as 5 bytes
            // Byte offsets: 0,3,4,7,9,11,12,15,20,21,22,25,28,30,31,34,39,44,45,47
            // This is complex. Let me use SameFrameExtended for safety.
            vec![StackFrame::AppendFrame {
                frame_type: 252, // 252 = append 1 local
                offset_delta: 43, // byte offset of target
                locals: vec![set_vtype],
            }],
        ));
    }

    // === static typeofValue(Object)String ===
    {
        let get_class_m = cp.add_method_ref(object_class_ref, "getClass", "()Ljava/lang/Class;").unwrap();
        let class_cls = cp.add_class("java/lang/Class").unwrap();
        let get_name_m = cp.add_method_ref(class_cls, "getName", "()Ljava/lang/String;").unwrap();
        let string_cls = cp.add_class("java/lang/String").unwrap();
        let contains_m = cp.add_method_ref(string_cls, "contains", "(Ljava/lang/CharSequence;)Z").unwrap();
        let lambda_str = cp.add_string("Lambda").unwrap();

        let typeof_value_code = vec![
            /* 0 */ Instruction::Aload_0,
            /* 1 */ Instruction::Ifnull(17),
            
            /* 2 */ Instruction::Aload_0,
            /* 3 */ Instruction::Instanceof(cp.add_class("java/lang/Double").unwrap()),
            /* 4 */ Instruction::Ifne(19),
            
            /* 5 */ Instruction::Aload_0,
            /* 6 */ Instruction::Instanceof(cp.add_class("java/lang/String").unwrap()),
            /* 7 */ Instruction::Ifne(21),
            
            /* 8 */ Instruction::Aload_0,
            /* 9 */ Instruction::Instanceof(cp.add_class("java/lang/Boolean").unwrap()),
            /* 10 */ Instruction::Ifne(23),
            
            /* 11 */ Instruction::Aload_0,
            /* 12 */ Instruction::Invokevirtual(get_class_m),
            /* 13 */ Instruction::Invokevirtual(get_name_m),
            /* 14 */ Instruction::Ldc_w(lambda_str),
            /* 15 */ Instruction::Invokevirtual(contains_m),
            /* 16 */ Instruction::Ifne(25),
            
            /* 17 */ Instruction::Ldc_w(cp.add_string("object").unwrap()),
            /* 18 */ Instruction::Areturn,
            
            /* 19 */ Instruction::Ldc_w(cp.add_string("number").unwrap()),
            /* 20 */ Instruction::Areturn,
            
            /* 21 */ Instruction::Ldc_w(cp.add_string("string").unwrap()),
            /* 22 */ Instruction::Areturn,
            
            /* 23 */ Instruction::Ldc_w(cp.add_string("boolean").unwrap()),
            /* 24 */ Instruction::Areturn,
            
            /* 25 */ Instruction::Ldc_w(cp.add_string("function").unwrap()),
            /* 26 */ Instruction::Areturn,
        ];

        methods.push(make_method(
            &mut cp,
            code_attr,
            "typeofValue",
            "(Ljava/lang/Object;)Ljava/lang/String;",
            MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            typeof_value_code,
            4,
            1,
        ));
    }

    // === static isTruthy(Object)Z ===
    {
        let code = vec![
            Instruction::Aload_0,
            Instruction::Ifnull(33),
            Instruction::Aload_0,
            Instruction::Instanceof(boolean_class),
            Instruction::Ifeq(9),
            Instruction::Aload_0,
            Instruction::Checkcast(boolean_class),
            Instruction::Invokevirtual(boolean_boolean_value),
            Instruction::Ireturn,
            Instruction::Aload_0,
            Instruction::Instanceof(double_class),
            Instruction::Ifeq(25),
            Instruction::Aload_0,
            Instruction::Checkcast(double_class),
            Instruction::Invokevirtual(double_double_value),
            Instruction::Dstore(1),
            Instruction::Dload(1),
            Instruction::Dconst_0,
            Instruction::Dcmpg,
            Instruction::Ifeq(33),
            Instruction::Dload(1),
            Instruction::Invokestatic(double_is_nan),
            Instruction::Ifne(33),
            Instruction::Iconst_1,
            Instruction::Ireturn,
            Instruction::Aload_0,
            Instruction::Instanceof(string_class),
            Instruction::Ifeq(35),
            Instruction::Aload_0,
            Instruction::Checkcast(string_class),
            Instruction::Invokevirtual(string_is_empty),
            Instruction::Ifne(33),
            Instruction::Goto(35),
            Instruction::Iconst_0,
            Instruction::Ireturn,
            Instruction::Iconst_1,
            Instruction::Ireturn,
        ];
        
        methods.push(make_method(
            &mut cp,
            code_attr,
            "isTruthy",
            "(Ljava/lang/Object;)Z",
            MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            code,
            4,
            3,
        ));
    }

    // === equals(Object)Z ===
    methods.push(make_method(
        &mut cp,
        code_attr,
        "equals",
        "(Ljava/lang/Object;)Z",
        MethodAccessFlags::PUBLIC,
        vec![
            Instruction::Aload_0,
            Instruction::Aload_1,
            Instruction::If_acmpne(5),
            Instruction::Iconst_1,
            Instruction::Ireturn,
            Instruction::Iconst_0,
            Instruction::Ireturn,
        ],
        2,
        2,
    ));

    // === hashCode()I ===
    methods.push(make_method(
        &mut cp,
        code_attr,
        "hashCode",
        "()I",
        MethodAccessFlags::PUBLIC,
        vec![
            Instruction::Aload_0,
            Instruction::Invokestatic(identity_hash_code),
            Instruction::Ireturn,
        ],
        1,
        1,
    ));

    let class_file = ClassFile {
        version: Version::Java21 { minor: 0 },
        constant_pool: cp,
        access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
        this_class,
        super_class,
        interfaces: Vec::new(),
        fields,
        methods,
        attributes: Vec::new(),
    };

    let mut buf = Vec::new();
    class_file.to_bytes(&mut buf).unwrap();
    (TS_OBJECT_PATH.to_string(), buf)
}

fn make_method(
    cp: &mut ConstantPool,
    code_attr_name: u16,
    name: &str,
    descriptor: &str,
    flags: MethodAccessFlags,
    code: Vec<Instruction>,
    max_stack: u16,
    max_locals: u16,
) -> Method {
    make_method_with_frames(cp, code_attr_name, name, descriptor, flags, code, max_stack, max_locals, vec![])
}

fn make_method_with_frames(
    cp: &mut ConstantPool,
    code_attr_name: u16,
    name: &str,
    descriptor: &str,
    flags: MethodAccessFlags,
    code: Vec<Instruction>,
    max_stack: u16,
    max_locals: u16,
    frames: Vec<StackFrame>,
) -> Method {
    make_method_with_frames_for_class(cp, code_attr_name, name, descriptor, flags, code, max_stack, max_locals, frames, TS_OBJECT_PATH)
}

fn make_method_for_class(
    cp: &mut ConstantPool,
    code_attr_name: u16,
    name: &str,
    descriptor: &str,
    flags: MethodAccessFlags,
    code: Vec<Instruction>,
    max_stack: u16,
    max_locals: u16,
    class_path: &str,
) -> Method {
    make_method_with_frames_for_class(cp, code_attr_name, name, descriptor, flags, code, max_stack, max_locals, vec![], class_path)
}

fn make_method_with_frames_for_class(
    cp: &mut ConstantPool,
    code_attr_name: u16,
    name: &str,
    descriptor: &str,
    flags: MethodAccessFlags,
    code: Vec<Instruction>,
    max_stack: u16,
    max_locals: u16,
    frames: Vec<StackFrame>,
    class_path: &str,
) -> Method {
    use crate::codegen::compiler::bytecode_emitter::generate_stack_map_table;
    use ristretto_classfile::attributes::VerificationType;
    
    let name_idx = cp.add_utf8(name).unwrap();
    let desc_idx = cp.add_utf8(descriptor).unwrap();
    
    let mut code_attributes = vec![];
    if !frames.is_empty() {
        let smt_name = cp.add_utf8("StackMapTable").unwrap();
        code_attributes.push(Attribute::StackMapTable {
            name_index: smt_name,
            frames,
        });
    } else {
        let mut initial_locals = vec![];
        if !flags.contains(MethodAccessFlags::STATIC) {
            initial_locals.push(VerificationType::Object {
                cpool_index: cp.add_class(class_path).unwrap_or(0),
            });
        }

        let args_str = descriptor.split(')').next().unwrap().trim_start_matches('(');
        let mut i = 0;
        let chars: Vec<char> = args_str.chars().collect();
        while i < chars.len() {
            match chars[i] {
                'L' => {
                    let start = i + 1;
                    let mut end = start;
                    while chars[end] != ';' { end += 1; }
                    let class_name: String = chars[start..end].iter().collect();
                    initial_locals.push(VerificationType::Object {
                        cpool_index: cp.add_class(&class_name).unwrap_or(0),
                    });
                    i = end + 1;
                },
                'Z' | 'B' | 'C' | 'S' | 'I' => {
                    initial_locals.push(VerificationType::Integer);
                    i += 1;
                },
                'F' => {
                    initial_locals.push(VerificationType::Float);
                    i += 1;
                },
                'D' => {
                    initial_locals.push(VerificationType::Double);
                    initial_locals.push(VerificationType::Top);
                    i += 1;
                },
                'J' => {
                    initial_locals.push(VerificationType::Long);
                    initial_locals.push(VerificationType::Top);
                    i += 1;
                },
                '[' => {
                    let start = i;
                    let mut end = start + 1;
                    while chars[end] == '[' { end += 1; }
                    if chars[end] == 'L' {
                        while chars[end] != ';' { end += 1; }
                    }
                    let array_desc: String = chars[start..=end].iter().collect();
                    initial_locals.push(VerificationType::Object {
                        cpool_index: cp.add_class(&array_desc).unwrap_or(0),
                    });
                    i = end + 1;
                },
                _ => { i += 1; }
            }
        }
        
        code_attributes = generate_stack_map_table(cp, &code, &[], initial_locals);
    }
    
    Method {
        access_flags: flags,
        name_index: name_idx,
        descriptor_index: desc_idx,
        attributes: vec![Attribute::Code {
            name_index: code_attr_name,
            max_stack,
            max_locals,
            code,
            exceptions: vec![],
            attributes: code_attributes,
        }],
    }
}

pub fn generate_ts_error_class() -> (String, Vec<u8>) {
    let mut cp = ConstantPool::default();
    
    let ts_error_path = "com/tsdroid/runtime/TsError";
    let super_path = "java/lang/RuntimeException";
    
    let this_class = cp.add_class(ts_error_path).unwrap();
    let super_class = cp.add_class(super_path).unwrap();
    let string_class = cp.add_class("java/lang/String").unwrap();
    
    let code_attr = cp.add_utf8("Code").unwrap();
    
    let string_value_of = cp.add_method_ref(
        string_class,
        "valueOf",
        "(Ljava/lang/Object;)Ljava/lang/String;",
    ).unwrap();
    
    let super_init = cp.add_method_ref(
        super_class,
        "<init>",
        "(Ljava/lang/String;)V",
    ).unwrap();
    
    let get_message_method_ref = cp.add_method_ref(
        this_class,
        "getMessage",
        "([Ljava/lang/Object;)Ljava/lang/String;",
    ).unwrap();

    // 1. Private static helper getMessage([Ljava/lang/Object;)Ljava/lang/String;
    let get_message_code = vec![
        Instruction::Aload_0,
        Instruction::Ifnull(10), // label_null
        Instruction::Aload_0,
        Instruction::Arraylength,
        Instruction::Ifeq(10), // label_null
        Instruction::Aload_0,
        Instruction::Iconst_0,
        Instruction::Aaload,
        Instruction::Invokestatic(string_value_of),
        Instruction::Areturn,
        // label_null (index 10)
        Instruction::Aconst_null,
        Instruction::Areturn,
    ];

    let get_message_method = make_method_for_class(
        &mut cp,
        code_attr,
        "getMessage",
        "([Ljava/lang/Object;)Ljava/lang/String;",
        MethodAccessFlags::PRIVATE | MethodAccessFlags::STATIC,
        get_message_code,
        2,
        1,
        ts_error_path,
    );

    // 2. Public constructor ([Ljava/lang/Object;)V
    let constructor_code = vec![
        Instruction::Aload_0,
        Instruction::Aload_1,
        Instruction::Invokestatic(get_message_method_ref),
        Instruction::Invokespecial(super_init),
        Instruction::Return,
    ];
    
    let constructor = make_method_for_class(
        &mut cp,
        code_attr,
        "<init>",
        "([Ljava/lang/Object;)V",
        MethodAccessFlags::PUBLIC,
        constructor_code,
        2,
        2,
        ts_error_path,
    );

    let constructor_obj_code = vec![
        Instruction::Aload_0,
        Instruction::Aload_1,
        Instruction::Invokestatic(string_value_of),
        Instruction::Invokespecial(super_init),
        Instruction::Return,
    ];
    let constructor_obj = make_method_for_class(
        &mut cp,
        code_attr,
        "<init>",
        "(Ljava/lang/Object;)V",
        MethodAccessFlags::PUBLIC,
        constructor_obj_code,
        2,
        2,
        ts_error_path,
    );

    let constructor_void_code = vec![
        Instruction::Aload_0,
        Instruction::Aconst_null,
        Instruction::Invokespecial(super_init),
        Instruction::Return,
    ];
    let constructor_void = make_method_for_class(
        &mut cp,
        code_attr,
        "<init>",
        "()V",
        MethodAccessFlags::PUBLIC,
        constructor_void_code,
        2,
        1,
        ts_error_path,
    );
    
    let class_file = ClassFile {
        version: Version::Java21 { minor: 0 },
        constant_pool: cp,
        access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
        this_class,
        super_class,
        interfaces: Vec::new(),
        fields: Vec::new(),
        methods: vec![constructor, constructor_obj, constructor_void, get_message_method],
        attributes: Vec::new(),
    };
    
    let mut buf = Vec::new();
    class_file.to_bytes(&mut buf).unwrap();
    (ts_error_path.to_string(), buf)
}

pub fn generate_ts_generator_class() -> (String, Vec<u8>) {
    let mut cp = ConstantPool::default();
    
    let ts_gen_path = "com/tsdroid/runtime/TsGenerator";
    let super_path = "java/lang/Object";
    
    let this_class = cp.add_class(ts_gen_path).unwrap();
    let super_class = cp.add_class(super_path).unwrap();
    
    let iterator_class = cp.add_class("java/util/Iterator").unwrap();
    let iterable_class = cp.add_class("java/lang/Iterable").unwrap();
    let queue_class = cp.add_class("java/util/concurrent/LinkedBlockingQueue").unwrap();
    let object_class = cp.add_class("java/lang/Object").unwrap();
    let exc_class = cp.add_class("java/util/NoSuchElementException").unwrap();
    
    let code_attr = cp.add_utf8("Code").unwrap();

    let queue_field_name = cp.add_utf8("queue").unwrap();
    let queue_field_desc = cp.add_utf8("Ljava/util/concurrent/LinkedBlockingQueue;").unwrap();
    let next_field_name = cp.add_utf8("nextVal").unwrap();
    let next_field_desc = cp.add_utf8("Ljava/lang/Object;").unwrap();
    let finished_field_name = cp.add_utf8("hasFinished").unwrap();
    let finished_field_desc = cp.add_utf8("Z").unwrap();
    let cached_field_name = cp.add_utf8("hasCached").unwrap();
    let cached_field_desc = cp.add_utf8("Z").unwrap();
    let sentinel_field_name = cp.add_utf8("SENTINEL").unwrap();
    let sentinel_field_desc = cp.add_utf8("Ljava/lang/Object;").unwrap();

    let fields = vec![
        Field {
            access_flags: FieldAccessFlags::PUBLIC,
            name_index: queue_field_name,
            descriptor_index: queue_field_desc,
            field_type: FieldType::Object("java/util/concurrent/LinkedBlockingQueue".to_string()),
            attributes: vec![],
        },
        Field {
            access_flags: FieldAccessFlags::PUBLIC,
            name_index: next_field_name,
            descriptor_index: next_field_desc,
            field_type: FieldType::Object("java/lang/Object".to_string()),
            attributes: vec![],
        },
        Field {
            access_flags: FieldAccessFlags::PUBLIC,
            name_index: finished_field_name,
            descriptor_index: finished_field_desc,
            field_type: FieldType::Base(BaseType::Boolean),
            attributes: vec![],
        },
        Field {
            access_flags: FieldAccessFlags::PUBLIC,
            name_index: cached_field_name,
            descriptor_index: cached_field_desc,
            field_type: FieldType::Base(BaseType::Boolean),
            attributes: vec![],
        },
        Field {
            access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            name_index: sentinel_field_name,
            descriptor_index: sentinel_field_desc,
            field_type: FieldType::Object("java/lang/Object".to_string()),
            attributes: vec![],
        },
    ];

    let obj_init = cp.add_method_ref(object_class, "<init>".to_string(), "()V".to_string()).unwrap();
    let sentinel_field_ref = cp.add_field_ref(this_class, "SENTINEL".to_string(), "Ljava/lang/Object;".to_string()).unwrap();
    
    let clinit_code = vec![
        Instruction::New(object_class),
        Instruction::Dup,
        Instruction::Invokespecial(obj_init),
        Instruction::Putstatic(sentinel_field_ref),
        Instruction::Return,
    ];
    let clinit = make_method_for_class(
        &mut cp,
        code_attr,
        "<clinit>",
        "()V",
        MethodAccessFlags::STATIC,
        clinit_code,
        2,
        0,
        ts_gen_path,
    );

    let super_init = cp.add_method_ref(super_class, "<init>".to_string(), "()V".to_string()).unwrap();
    let queue_field_ref = cp.add_field_ref(this_class, "queue".to_string(), "Ljava/util/concurrent/LinkedBlockingQueue;".to_string()).unwrap();
    
    let init_code = vec![
        Instruction::Aload_0,
        Instruction::Invokespecial(super_init),
        Instruction::Aload_0,
        Instruction::Aload_1,
        Instruction::Putfield(queue_field_ref),
        Instruction::Return,
    ];
    let constructor = make_method_for_class(
        &mut cp,
        code_attr,
        "<init>",
        "(Ljava/util/concurrent/LinkedBlockingQueue;)V",
        MethodAccessFlags::PUBLIC,
        init_code,
        2,
        2,
        ts_gen_path,
    );

    let iterator_code = vec![
        Instruction::Aload_0,
        Instruction::Areturn,
    ];
    let iterator_method = make_method_for_class(
        &mut cp,
        code_attr,
        "iterator",
        "()Ljava/util/Iterator;",
        MethodAccessFlags::PUBLIC,
        iterator_code,
        1,
        1,
        ts_gen_path,
    );

    let finished_field_ref = cp.add_field_ref(this_class, "hasFinished".to_string(), "Z".to_string()).unwrap();
    let cached_field_ref = cp.add_field_ref(this_class, "hasCached".to_string(), "Z".to_string()).unwrap();
    let next_field_ref = cp.add_field_ref(this_class, "nextVal".to_string(), "Ljava/lang/Object;".to_string()).unwrap();
    let take_method_ref = cp.add_method_ref(queue_class, "take".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
    
    let has_next_code = vec![
        Instruction::Aload_0,
        Instruction::Getfield(cached_field_ref),
        Instruction::Ifne(19),
        Instruction::Aload_0,
        Instruction::Getfield(finished_field_ref),
        Instruction::Ifne(24),
        Instruction::Aload_0,
        Instruction::Getfield(queue_field_ref),
        Instruction::Invokevirtual(take_method_ref),
        Instruction::Astore_1,
        Instruction::Aload_1,
        Instruction::Getstatic(sentinel_field_ref),
        Instruction::If_acmpeq(21),
        Instruction::Aload_0,
        Instruction::Aload_1,
        Instruction::Putfield(next_field_ref),
        Instruction::Aload_0,
        Instruction::Iconst_1,
        Instruction::Putfield(cached_field_ref),
        Instruction::Iconst_1,
        Instruction::Ireturn,
        Instruction::Aload_0,
        Instruction::Iconst_1,
        Instruction::Putfield(finished_field_ref),
        Instruction::Iconst_0,
        Instruction::Ireturn,
    ];
    let has_next_method = make_method_for_class(
        &mut cp,
        code_attr,
        "hasNext",
        "()Z",
        MethodAccessFlags::PUBLIC,
        has_next_code,
        2,
        2,
        ts_gen_path,
    );

    let exc_init = cp.add_method_ref(exc_class, "<init>".to_string(), "()V".to_string()).unwrap();
    let has_next_method_ref = cp.add_method_ref(this_class, "hasNext".to_string(), "()Z".to_string()).unwrap();
    
    let next_code = vec![
        Instruction::Aload_0,
        Instruction::Invokevirtual(has_next_method_ref),
        Instruction::Ifne(7),
        Instruction::New(exc_class),
        Instruction::Dup,
        Instruction::Invokespecial(exc_init),
        Instruction::Athrow,
        Instruction::Aload_0,
        Instruction::Iconst_0,
        Instruction::Putfield(cached_field_ref),
        Instruction::Aload_0,
        Instruction::Getfield(next_field_ref),
        Instruction::Areturn,
    ];
    let next_method = make_method_for_class(
        &mut cp,
        code_attr,
        "next",
        "()Ljava/lang/Object;",
        MethodAccessFlags::PUBLIC,
        next_code,
        2,
        1,
        ts_gen_path,
    );

    let next_method_ref = cp.add_method_ref(this_class, "next".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
    let ts_object_class = cp.add_class("com/tsdroid/runtime/TsObject").unwrap();
    let ts_object_init = cp.add_method_ref(ts_object_class, "<init>".to_string(), "()V".to_string()).unwrap();
    let set_property = cp.add_method_ref(ts_object_class, "setProperty".to_string(), "(Ljava/lang/String;Ljava/lang/Object;)V".to_string()).unwrap();
    let boolean_class = cp.add_class("java/lang/Boolean").unwrap();
    let true_field = cp.add_field_ref(boolean_class, "TRUE".to_string(), "Ljava/lang/Boolean;".to_string()).unwrap();
    let false_field = cp.add_field_ref(boolean_class, "FALSE".to_string(), "Ljava/lang/Boolean;".to_string()).unwrap();
    let value_str_idx = cp.add_string("value").unwrap();
    let done_str_idx = cp.add_string("done").unwrap();

    let next_bridge_code = vec![
        // Check hasNext()
        Instruction::Aload_0,
        Instruction::Invokevirtual(has_next_method_ref), // returns boolean
        Instruction::Ifeq(20), // if false, jump to done_block (offset 20)

        // 1. Create TsObject
        Instruction::New(ts_object_class),
        Instruction::Dup,
        Instruction::Invokespecial(ts_object_init),
        Instruction::Astore_2, // store TsObject in slot 2

        // 2. Call next() to get value
        Instruction::Aload_0,
        Instruction::Invokevirtual(next_method_ref), // returns Object
        Instruction::Astore_3, // store value in slot 3

        // 3. setProperty("value", value)
        Instruction::Aload_2,
        Instruction::Ldc_w(value_str_idx),
        Instruction::Aload_3,
        Instruction::Invokevirtual(set_property),

        // 4. setProperty("done", Boolean.FALSE)
        Instruction::Aload_2,
        Instruction::Ldc_w(done_str_idx),
        Instruction::Getstatic(false_field),
        Instruction::Invokevirtual(set_property),

        // 5. Return TsObject
        Instruction::Aload_2,
        Instruction::Areturn,

        // --- done_block --- (offset 20)
        // 1. Create TsObject
        Instruction::New(ts_object_class),
        Instruction::Dup,
        Instruction::Invokespecial(ts_object_init),
        Instruction::Astore_2,

        // 2. setProperty("value", null)
        Instruction::Aload_2,
        Instruction::Ldc_w(value_str_idx),
        Instruction::Aconst_null,
        Instruction::Invokevirtual(set_property),

        // 3. setProperty("done", Boolean.TRUE)
        Instruction::Aload_2,
        Instruction::Ldc_w(done_str_idx),
        Instruction::Getstatic(true_field),
        Instruction::Invokevirtual(set_property),

        // 4. Return TsObject
        Instruction::Aload_2,
        Instruction::Areturn,
    ];

    let next_bridge_method = make_method_for_class(
        &mut cp,
        code_attr,
        "next",
        "([Ljava/lang/Object;)Ljava/lang/Object;",
        MethodAccessFlags::PUBLIC,
        next_bridge_code,
        3,
        4,
        ts_gen_path,
    );

    let class_file = ClassFile {
        version: Version::Java21 { minor: 0 },
        constant_pool: cp,
        access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
        this_class,
        super_class,
        interfaces: vec![iterator_class, iterable_class],
        fields,
        methods: vec![clinit, constructor, iterator_method, has_next_method, next_method, next_bridge_method],
        attributes: Vec::new(),
    };
    
    let mut buf = Vec::new();
    class_file.to_bytes(&mut buf).unwrap();
    (ts_gen_path.to_string(), buf)
}

use crate::codegen::compiler::runtime_bytes;

pub fn generate_ts_runtime_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsRuntime".to_string(), runtime_bytes::TS_RUNTIME_BYTES.to_vec())
}

pub fn generate_ts_date_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsDate".to_string(), runtime_bytes::TS_DATE_BYTES.to_vec())
}

pub fn generate_ts_map_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsMap".to_string(), runtime_bytes::TS_MAP_BYTES.to_vec())
}

pub fn generate_ts_set_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsSet".to_string(), runtime_bytes::TS_SET_BYTES.to_vec())
}

pub fn generate_ts_weakmap_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsWeakMap".to_string(), runtime_bytes::TS_WEAK_MAP_BYTES.to_vec())
}

pub fn generate_ts_weakset_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsWeakSet".to_string(), runtime_bytes::TS_WEAK_SET_BYTES.to_vec())
}

pub fn generate_ts_json_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsJSON".to_string(), runtime_bytes::TS_JSON_BYTES.to_vec())
}

pub fn generate_ts_json_parser_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsJSON$Parser".to_string(), runtime_bytes::TS_JSON_PARSER_BYTES.to_vec())
}

pub fn generate_ts_regexp_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsRegExp".to_string(), runtime_bytes::TS_REGEXP_BYTES.to_vec())
}

pub fn generate_ts_response_class() -> (String, Vec<u8>) {
    ("com/tsdroid/runtime/TsResponse".to_string(), runtime_bytes::TS_RESPONSE_BYTES.to_vec())
}
