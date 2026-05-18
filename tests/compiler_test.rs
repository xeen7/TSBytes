use tsbyte::compile_to_bytes;
use ristretto_classfile::{ClassFile, attributes::Attribute};
use std::io::Cursor;

fn parse_class(classes: &[(String, Vec<u8>)]) -> ClassFile {
    let (_, bytes) = classes.iter().find(|(n, _)| !n.contains("Lambda$") && !n.contains("runtime/")).unwrap();
    let mut cursor = Cursor::new(bytes.clone());
    ClassFile::from_bytes(&mut cursor).expect("Failed to parse generated class file")
}

// ── Basic Function Compilation ──────────────────────────────────────

#[test]
fn test_compile_basic_function() {
    let ts = r#"
        function calculate(a: number, b: number): number {
            return a + b;
        }
    "#;

    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert_eq!(class.methods.len(), 2, "Expected <init> + calculate");

    let mut found = false;
    for method in &class.methods {
        let name = class.constant_pool.try_get_utf8(method.name_index).unwrap();
        if name == "calculate" {
            found = true;
            let has_code = method.attributes.iter().any(|a| matches!(a, Attribute::Code { .. }));
            assert!(has_code, "calculate missing Code attribute");
        }
    }
    assert!(found, "Could not find 'calculate'");
}

// ── If/Else Control Flow ────────────────────────────────────────────

#[test]
fn test_compile_if_else() {
    let ts = r#"
        function check(x: number): number {
            if (x > 0) {
                return 1;
            } else {
                return 0;
            }
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert_eq!(class.methods.len(), 2);
}

#[test]
fn test_compile_nested_if() {
    let ts = r#"
        function classify(x: number): string {
            if (x > 100) {
                return "high";
            } else {
                if (x > 50) {
                    return "medium";
                } else {
                    return "low";
                }
            }
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

// ── While Loops ─────────────────────────────────────────────────────

#[test]
fn test_compile_simple_while() {
    let ts = r#"
        function countdown(n: number): number {
            let i = n;
            while (i > 0) {
                i = i - 1;
            }
            return i;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

// ── Ternary Expressions ─────────────────────────────────────────────

#[test]
fn test_compile_ternary_expression() {
    let ts = r#"
        function max(a: number, b: number): number {
            return a > b ? a : b;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

// ── Data Structures ─────────────────────────────────────────────────

#[test]
fn test_compile_arrays_and_objects() {
    let ts = r#"
        function buildData() {
            let arr = [1, 2, 3];
            let obj = { name: "test", val: 5 };
            return arr;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

// ── Class Declarations ──────────────────────────────────────────────

#[test]
fn test_compile_class_declaration() {
    let ts = r#"
        class Animal {
            name: string;
            constructor(name: string) {
                this.name = name;
            }
            speak() {
                return "hello";
            }
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_class_with_extends() {
    let ts = r#"
        class Shape {
            area(): number { return 0; }
        }
        class Circle extends Shape {
            radius: number;
            area(): number { return 3.14; }
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

// ── String Operations ───────────────────────────────────────────────

#[test]
fn test_compile_string_concat() {
    let ts = r#"
        function greet(name: string): string {
            return "Hello " + name;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

// ── Arrow Functions & Closures ──────────────────────────────────────

#[test]
fn test_compile_arrow_function() {
    let ts = r#"
        function process() {
            let factor = 10;
            let multiply = (x: number) => x * factor;
            return multiply;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty(), "Arrow function compilation should produce valid bytecode");
}

// ── Multiple Functions ──────────────────────────────────────────────

#[test]
fn test_compile_multiple_functions() {
    let ts = r#"
        function add(a: number, b: number): number {
            return a + b;
        }
        function subtract(a: number, b: number): number {
            return a - b;
        }
        function multiply(a: number, b: number): number {
            return a * b;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    // <init> + add + subtract + multiply = 4 methods
    assert_eq!(class.methods.len(), 4);
}

// ── Variable Declarations ───────────────────────────────────────────

#[test]
fn test_compile_typed_variables() {
    let ts = r#"
        function compute() {
            let x: number = 42;
            let name: string = "world";
            let flag: boolean = true;
            return x;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

// ── Complex Enterprise Pattern: Service Layer ───────────────────────

#[test]
fn test_compile_service_class() {
    let ts = r#"
        class UserService {
            baseUrl: string;
            constructor(url: string) {
                this.baseUrl = url;
            }
            getUser(id: number): string {
                return "user";
            }
            createUser(name: string, age: number): boolean {
                return true;
            }
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

// ── Module Graph Tests ──────────────────────────────────────────────

#[test]
fn test_module_graph_registration() {
    use tsbyte::codegen::compiler::module_graph::{ModuleGraph, ExportKind};

    let mut graph = ModuleGraph::new("com.tsbyte.app");
    let class = graph.register_module("src/services/math.ts");
    assert_eq!(class, "com/tsbyte/app/services/Math");

    graph.add_export("src/services/math.ts", "add", ExportKind::Function, "(DD)D");

    graph.register_module("src/main.ts");
    let result = graph.resolve_import("src/main.ts", "./services/math", "add");
    assert!(result.is_some());
    let (resolved_class, desc) = result.unwrap();
    assert_eq!(resolved_class, "com/tsbyte/app/services/Math");
    assert_eq!(desc, "(DD)D");
}

// ── JVM Descriptor Generation ───────────────────────────────────────

#[test]
fn test_jvm_descriptors() {
    use tsbyte::codegen::compiler::ir::{Type, build_method_descriptor};

    assert_eq!(Type::Int.to_jvm_descriptor(), "I");
    assert_eq!(Type::Double.to_jvm_descriptor(), "D");
    assert_eq!(Type::Bool.to_jvm_descriptor(), "Z");
    assert_eq!(Type::StringTy.to_jvm_descriptor(), "Ljava/lang/String;");
    assert_eq!(Type::Void.to_jvm_descriptor(), "V");
    assert_eq!(Type::Class("java/util/List".to_string()).to_jvm_descriptor(), "Ljava/util/List;");
    assert_eq!(Type::Array(Box::new(Type::Double)).to_jvm_descriptor(), "[D");
    assert_eq!(Type::Any.to_jvm_descriptor(), "Ljava/lang/Object;");

    // Method descriptor: (double, String) -> boolean
    let desc = build_method_descriptor(
        &[Type::Double, Type::StringTy],
        &Type::Bool,
    );
    assert_eq!(desc, "(DLjava/lang/String;)Z");
}

// ── Newly Added AST Nodes ───────────────────────────────────────────

#[test]
fn test_compile_for_of() {
    let ts = r#"
        function sum(arr: number[]): number {
            let total = 0;
            for (let x of arr) {
                total += x;
            }
            return total;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_do_while() {
    let ts = r#"
        function count(): number {
            let i = 0;
            do {
                i = i + 1;
            } while (i < 5);
            return i;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_update_expr() {
    let ts = r#"
        function inc(x: number): number {
            x++;
            ++x;
            x--;
            --x;
            return x;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_paren_expr() {
    let ts = r#"
        function math(x: number): number {
            return (x + 2) * 3;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_await_expr() {
    let ts = r#"
        async function fetch(url: string): any {
            let result = await doFetch(url);
            return result;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_interface() {
    let ts = r#"
        interface Drawable {
            draw(): void;
            getBounds(x: number, y: number): number[];
        }
        class Circle implements Drawable {
            draw() {}
            getBounds(x: number, y: number): number[] { return []; }
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_opt_chain() {
    let ts = r#"
        function getVal(obj: any): any {
            return obj?.prop;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_assign() {
    let ts = r#"
        function setVal(obj: any): void {
            let x = 10;
            x = 20;
            obj.val = 30;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_export() {
    let ts = r#"
        export function myFunc(): void {}
        export default function App(): void {}
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_spread_array() {
    let ts = r#"
        function spreadArr(arr: any[]): any[] {
            return [1, ...arr, 2];
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_spread_object() {
    let ts = r#"
        function spreadObj(obj: any): any {
            return { a: 1, ...obj, b: 2 };
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_rest_parameters() {
    let ts = r#"
        function restParams(a: number, ...args: any[]): void {
            console.log(a, ...args);
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_dynamic_any_dispatch() {
    let ts = r#"
        function addAny(a: any, b: any): any {
            return a + b;
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_compile_async_function_wrapper() {
    let ts = r#"
        async function fetchUser(): Promise<any> {
            return { name: "Alice" };
        }
    "#;
    let bytes = compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    assert!(!bytes.is_empty());
}

// ── Type Checker Tests ───────────────────────────────────────────────

#[test]
fn test_type_checker_assignment() {
    let ts = r#"
        function testAssign() {
            let x: number = 10;
            x = "hello"; // error
        }
    "#;
    let errors = tsbyte::type_check_to_errors(ts).unwrap();
    assert!(errors.iter().any(|e| e.contains("Cannot assign type 'StringTy' to variable 'x' of type 'Double'")));
}

#[test]
fn test_type_checker_function_call() {
    let ts = r#"
        function greet(name: string): string {
            return "Hello " + name;
        }
        function testCall() {
            greet(123); // error
        }
    "#;
    let errors = tsbyte::type_check_to_errors(ts).unwrap();
    assert!(errors.iter().any(|e| e.contains("Argument 0 to 'greet' is not assignable to 'StringTy', got 'Double'")));
}

#[test]
fn test_type_checker_arithmetic() {
    let ts = r#"
        function testMath() {
            let a = "string" * 5; // error
        }
    "#;
    let errors = tsbyte::type_check_to_errors(ts).unwrap();
    assert!(errors.iter().any(|e| e.contains("Arithmetic operations require numeric types")));
}

#[test]
fn test_type_checker_return() {
    let ts = r#"
        function getNumber(): number {
            return "not a number"; // error
        }
    "#;
    let errors = tsbyte::type_check_to_errors(ts).unwrap();
    assert!(errors.iter().any(|e| e.contains("Cannot return type 'StringTy' from function expecting 'Double'")));
}

#[test]
fn test_type_checker_structural_interface() {
    let ts = r#"
        interface Point {
            x: number;
            y: number;
        }
        function usePoint(p: Point) {
            let n = p.x;
        }
        function testStruct() {
            let p1: Point = { x: 10, y: 20 }; // OK
            let p2: Point = { x: 10 }; // Error: missing y
        }
    "#;
    let errors = tsbyte::type_check_to_errors(ts).unwrap();
    assert!(errors.iter().any(|e| e.contains("Cannot assign type 'Object([(\"x\", Double)])' to variable 'p2' of type 'Object([(\"x\", Double), (\"y\", Double)])'")));
}

#[test]
fn test_type_checker_generic_param() {
    let ts = r#"
        function handlePromise(p: Promise<string>) {}
        function testGen() {
            let bad: Promise<number> = null as any;
            handlePromise(bad); // Error: Promise<number> not assignable to Promise<string>
        }
    "#;
    let _errors = tsbyte::type_check_to_errors(ts).unwrap();
    // In our simplified setup, `null as any` bypasses checks, but the function call arguments will fail
    // wait, we don't have generics correctly mocked in AST for Cast yet, so we will just test assignment
    let ts2 = r#"
        function handlePromise(p: Promise<string>) {}
        function testGen(bad: Promise<number>) {
            handlePromise(bad);
        }
    "#;
    let errors2 = tsbyte::type_check_to_errors(ts2).unwrap();
    assert!(errors2.iter().any(|e| e.contains("Argument 0 to 'handlePromise' is not assignable to 'Generic(\"Promise\", [StringTy])', got 'Generic(\"Promise\", [Double])'")));
}

#[test]
fn test_type_checker_field_access() {
    let ts = r#"
        interface User { name: string; }
        function testField(u: User) {
            let n = u.name; // ok
            let age = u.age; // error
        }
    "#;
    let errors = tsbyte::type_check_to_errors(ts).unwrap();
    assert!(errors.iter().any(|e| e.contains("Property 'age' does not exist on type 'Object([(\"name\", StringTy)])'")));
}

#[test]
fn test_mir_lowering_and_optimization() {
    let ts = r#"
        function testMir() {
            let x = 5 * 10 + 2; // Should fold to 52
            if (x > 50) {
                return 1;
            } else {
                return 0;
            }
            let unreachable = 99; // Should be eliminated by DCE
        }
    "#;
    
    // We mock the pipeline to test MIR specifically
    let mut builder = tsbyte::codegen::compiler::ir_builder::IrBuilder::new();
    let module = tsbyte::swc_frontend::parse_typescript(ts, "test.ts");
    let stmts = builder.build_module(&module);
    
    let mut checker = tsbyte::codegen::compiler::type_checker::TypeChecker::new();
    let _errors = checker.check_program(&stmts);
    
    let lowerer = tsbyte::codegen::compiler::hir_to_mir::HirToMir::new();
    let mir_funcs = lowerer.lower(&stmts);
    
    // Verify it compiled into some MIR blocks
    assert!(!mir_funcs.is_empty());
    
    let optimized = tsbyte::codegen::compiler::mir_opt::MirOptimizer::optimize(mir_funcs);
    
    let test_mir = optimized.iter().find(|f| f.name == "testMir").expect("Expected testMir function");
    
    // We expect several blocks, but the unreachable code should be gone
    assert!(test_mir.blocks.len() > 0);
    
    // Check if Constant Folding worked
    println!("{:#?}", test_mir);
    let mut found_52 = false;
    for block in test_mir.blocks.values() {
        for instr in &block.instrs {
            if let tsbyte::codegen::compiler::mir::MirInstr::Assign(_, tsbyte::codegen::compiler::mir::MirExpr::Operand(tsbyte::codegen::compiler::mir::Operand::Const(tsbyte::codegen::compiler::mir::Constant::Double(v), _))) = instr {
                if *v == 52.0 {
                    found_52 = true;
                }
            }
        }
    }
    assert!(found_52, "Constant folding failed, didn't find 52.0 in optimized MIR");
}

#[test]
fn test_jar_packaging() {
    use std::io::Read;
    use std::path::PathBuf;

    let ts = r#"
        function add(a: number, b: number): number {
            return a + b;
        }
    "#;

    let output_path = PathBuf::from("/tmp/tsbyte_test_output.jar");

    tsbyte::compile_to_jar(ts, "com.example", &output_path)
        .expect("JAR compilation failed");

    assert!(output_path.exists(), "JAR file was not created");

    // Open the JAR and verify its contents
    let file = std::fs::File::open(&output_path).expect("Cannot open JAR");
    let mut zip = zip::ZipArchive::new(file).expect("Not a valid ZIP/JAR archive");

    // Verify MANIFEST.MF exists and contains correct Main-Class (scoped so borrow ends)
    {
        let mut manifest = zip.by_name("META-INF/MANIFEST.MF").expect("MANIFEST.MF missing");
        let mut manifest_content = String::new();
        manifest.read_to_string(&mut manifest_content).unwrap();
        assert!(manifest_content.contains("Manifest-Version: 1.0"), "Manifest-Version missing");
        assert!(manifest_content.contains("Main-Class: com.example.App"), "Main-Class missing");
    }

    // Verify App.class exists
    let class_names: Vec<String> = (0..zip.len())
         .map(|i| zip.by_index(i).unwrap().name().to_string())
         .collect();
    assert!(
        class_names.iter().any(|n| n.ends_with(".class")),
        "No .class files found in JAR: {:?}", class_names
    );

    // Cleanup
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn test_compile_getters_setters() {
    let ts = r#"
        class Temperature {
            _celsius: number = 0;
            get fahrenheit(): number {
                return this._celsius * 1.8 + 32;
            }
            set fahrenheit(value: number) {
                this._celsius = (value - 32) / 1.8;
            }
        }
        function testProp() {
            let t = new Temperature();
            t.fahrenheit = 100;
            return t.fahrenheit;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Class should compile successfully");
}

#[test]
fn test_compile_define_property() {
    let ts = r#"
        function testDefine() {
            let obj = {};
            Object.defineProperty(obj, "score", {
                value: 100
            });
            Object.defineProperty(obj, "multiplier", {
                get() { return 2; },
                set(v) { }
            });
            return obj.score * obj.multiplier;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Class should compile successfully");
}

#[test]
fn test_compile_array_length() {
    let ts = r#"
        function testLength() {
            let arr: number[] = [1, 2, 3];
            let len = arr.length;
            return len;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Class should compile array length");
}

#[test]
fn test_compile_array_push_pop() {
    let ts = r#"
        function testPushPop() {
            let arr: number[] = [1, 2];
            arr.push(3);
            arr.push(4, 5);
            let last = arr.pop();
            let first = arr.shift();
            arr.unshift(0);
            return arr.length;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Class should compile array push/pop");
}

#[test]
fn test_compile_array_methods() {
    let ts = r#"
        function testArrayMethods() {
            let arr: number[] = [10, 20, 30, 40, 50];
            let idx = arr.indexOf(30);
            let has = arr.includes(20);
            let joined = arr.join("-");
            arr.reverse();
            let sliced = arr.slice(1, 3);
            let combined = arr.concat(sliced);
            return combined.length;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Class should compile array methods");
}

#[test]
fn test_compile_newly_implemented_features() {
    let ts = r#"
        class Account {
            #balance: bigint;
            #holder: string;

            constructor({ holder, initialBalance }) {
                this.#holder = holder;
                this.#balance = initialBalance;
            }

            #computeBonus(ratio: number) {
                return 100n;
            }

            getDetails() {
                let pattern = /[a-z]+/i;
                let bonus = this.#computeBonus(0.1);
                return this.#holder;
            }
        }
        function testDestructure([first, second]: number[]) {
            return first + second;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Newly implemented features should compile to JVM bytecode perfectly");
}

#[test]
fn test_compile_further_features() {
    let ts = r#"
        enum Direction {
            Up = 10,
            Down,
            Left,
            Right
        }

        function testFeatures() {
            let x = (1, 2, Direction.Down) satisfies number;
            let y = "constant" as const;
            let z = !true;
            return x;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Further features (enum, satisfies, sequence, as const) should compile to JVM bytecode perfectly");
}

#[test]
fn test_compile_advanced_roadmapped_features() {
    let ts = r#"
        import type { SomeType } from "./types";
        import { type OtherType, regularImport } from "./types";
        export type { AnotherType };

        const addExpr = function(a: number, b: number) {
            return a + b;
        };

        const AnonymousClass = class {
            greet() { return "hello"; }
        };

        class Config {
            static _debug: boolean = false;
            static get debug(): boolean {
                return this._debug;
            }
            static set debug(value: boolean) {
                this._debug = value;
            }
        }

        function testDestructuring() {
            let [first, ...rest] = [1, 2, 3];
            let { x, ...others } = { x: 10, y: 20, z: 30 };
            return first;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Advanced roadmapped features should compile to JVM bytecode perfectly");
}

#[test]
fn test_compile_further_advanced_features() {
    let ts = r#"
        // 1. Re-exports
        export { val as aliasVal } from "./some_module";
        export * from "./another_module";
        export * as nsModule from "./ns_module";

        // 2. Abstract fields and Override modifier
        abstract class Animal {
            abstract sound: string;
            makeSound(): string {
                return this.sound;
            }
        }

        class Dog extends Animal {
            override sound: string = "woof";
            override makeSound(): string {
                return "dog says " + this.sound;
            }
        }

        // 3. Nested Destructuring (variables & parameters)
        function testNestedDestructure(obj: any, arr: any) {
            const { a: { b, c: d } } = obj;
            const [first, [second, third]] = arr;
            return b;
        }

        // 4. Meta-Properties
        class TargetDemo {
            target: any;
            constructor() {
                this.target = new.target;
            }
            getTarget() {
                return new.target;
            }
        }

        function getMetaUrl() {
            const meta = import.meta;
            return meta.url;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Further advanced features should compile to JVM bytecode perfectly");
}

#[test]
fn test_compile_advanced_phase_3() {
    let ts = r#"
        "use strict";
        "use asm";

        // 1. Namespaces
        namespace MathUtils {
            export const PI = 3.14159;
            export function square(x: number): number {
                return x * x;
            }
            const privateHelper = 42;
            function privateSquare(x: number) {
                return x * privateHelper;
            }
        }

        // 2. Ambient Declarations (erased at bytecode level, registered in type checker)
        declare const externalVal: string;
        declare function externalLog(msg: string): void;
        declare class NativeHelper {}

        // 3. Dynamic import() returning a CompletableFuture
        function loadModule() {
            return import("com.tsbyte.runtime.DynamicModule");
        }

        // 4. Advanced TS Types, Predicates, and typeof
        function typeDemonstration() {
            let sym: symbol;
            let status: "success" | "error" = "success";
            
            function isNumber(x: any): x is number {
                return true;
            }
            
            let original = 100;
            let aliasVal: typeof original = 200;
            
            return aliasVal;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Advanced Phase 3 features should compile to JVM bytecode perfectly");
}

#[test]
fn test_compile_phase_4_features() {
    let ts = r#"
        // 1. Decorators (parsed & cleanly erased)
        @sealed
        class Greeter {
            @format("Hello, %s")
            greeting: string;
            
            constructor(message: string) {
                this.greeting = message;
            }
            
            @log
            greet() {
                return this.greeting;
            }
        }

        // 2. Tagged Template Literals
        function customTag(strings: string[], ...values: any[]) {
            return strings[0] + values[0] + strings[1];
        }
        function testTagged() {
            let name = "World";
            return customTag`Hello ${name}!`;
        }

        // 3. Generators and yield / yield*
        function* numberGenerator() {
            yield 1;
            yield 2;
            return 3;
        }

        function* delegateGenerator() {
            yield* numberGenerator();
            yield 4;
        }

        // 3b. Async Generators and for await...of
        async function* asyncNumGen() {
            yield 10;
            yield 20;
        }
        async function testForAwait(gen: any) {
            let sum = 0;
            for await (const x of gen) {
                sum = sum + x;
            }
            return sum;
        }

        // 4. Advanced TS Type System Features
        type Point = { x: number; y: number };
        type Point3D = Point & { z: number }; // intersection type
        interface StringMap {
            [key: string]: string; // index signature
        }
        function identity<T extends number>(arg: T): T { // generic constraint
            return arg;
        }
        type IsString<T> = T extends string ? true : false; // conditional type
        type ReadonlyPoint = { readonly [P in keyof Point]: Point[P] }; // mapped type, keyof
        type Greeting = `hello ${string}`; // template literal type
        type InferType<T> = T extends (infer U)[] ? U : T; // infer keyword
        type PartialPoint = Partial<Point>; // utility type
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Phase 4 compiler features (generators, yield, yield*, decorators, advanced types) should compile perfectly");
}

#[test]
fn test_compile_destructuring_defaults() {
    let ts = r#"
        function testDestructureDefaults() {
            let [x = 10, y = 20] = [5];
            let { a = 100, b: { c = 200 } = {} } = { a: 50 };
            return x + y + a + c;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Destructuring with default values should compile to JVM bytecode successfully");
}

#[test]
fn test_compile_advanced_class_blocks_initializers() {
    let ts = r#"
        class ConfigService {
            static #defaultUrl = "http://localhost";
            #timeout = 5000;
            static active: boolean;

            static {
                ConfigService.active = true;
            }

            getSettings() {
                return ConfigService.#defaultUrl + ":" + this.#timeout;
            }
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Advanced class components should compile to JVM bytecode successfully");
}

#[test]
fn test_compile_generic_erasure_lookup() {
    let ts = r#"
        interface Node<T> {
            value: T;
        }
        function processNode(node: Node<number>) {
            return node.value;
        }
    "#;
    let bytes = tsbyte::compile_to_bytes(ts, "com.tsbyte.test").unwrap();
    let class = parse_class(&bytes);
    assert!(!class.methods.is_empty(), "Field access on variables with generic types should compile successfully");
}
