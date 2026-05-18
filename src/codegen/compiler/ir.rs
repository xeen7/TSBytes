/// High-level Intermediate Representation for the TSDroid compiler.
/// This IR is the canonical representation between the SWC AST frontend
/// and the JVM bytecode backend. Every TypeScript construct must be
/// expressible through these types.

// ── Type System ──────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    Double,
    Bool,
    StringTy,
    Void,
    Class(String),
    Array(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    /// Union types (e.g. `string | number`) — boxed to Object on JVM
    Union(Vec<Type>),
    /// Nullable wrapper — `T | null`
    Nullable(Box<Type>),
    /// Structural object type: `{ id: number, name: string }`
    Object(Vec<(String, Type)>),
    /// Generic type: `Promise<string>`
    Generic(String, Vec<Type>),
    Any,
    Null,
    Undefined,
    Unknown,
    /// Never type — for exhaustiveness checks (e.g. switch default)
    Never,
    /// Tuple type: `[string, number]`
    Tuple(Vec<Type>),
    BigInt,
    RegExp,
    Symbol,
}

impl Type {
    /// Convert a Type to its JVM descriptor string.
    /// This is critical for correct method signatures and field types.
    pub fn to_jvm_descriptor(&self) -> String {
        match self {
            Type::Int => "I".to_string(),
            Type::Double => "D".to_string(),
            Type::Bool => "Z".to_string(),
            Type::StringTy => "Ljava/lang/String;".to_string(),
            Type::Void => "V".to_string(),
            Type::Class(name) => format!("L{};", name.replace('.', "/")),
            Type::Array(inner) => format!("[{}", inner.to_jvm_descriptor()),
            Type::Function(_, _) => "Ljava/lang/Object;".to_string(), // functional interface
            Type::BigInt => "Ljava/math/BigInteger;".to_string(),
            Type::RegExp => "Ljava/util/regex/Pattern;".to_string(),
            Type::Symbol => "Ljava/lang/Object;".to_string(),
            Type::Union(_) | Type::Any | Type::Null | Type::Undefined | Type::Unknown
            | Type::Nullable(_) | Type::Never | Type::Tuple(_)
            | Type::Object(_) | Type::Generic(_, _) => "Ljava/lang/Object;".to_string(),
        }
    }

    /// Whether this type occupies 2 slots on the JVM operand stack (double, long)
    pub fn is_wide(&self) -> bool {
        matches!(self, Type::Double)
    }

    /// Whether this type is a JVM primitive (not an object reference)
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Int | Type::Double | Type::Bool)
    }

    /// Whether this type needs boxing to be passed as Object
    pub fn needs_boxing(&self) -> bool {
        self.is_primitive()
    }

    /// Checks if a value of `self` type can be assigned to a variable of `target` type
    pub fn is_assignable_to(&self, target: &Type) -> bool {
        if self == target {
            return true;
        }
        if matches!(target, Type::Any) || matches!(self, Type::Any) || matches!(target, Type::Unknown) || matches!(self, Type::Unknown) {
            return true;
        }
        match (self, target) {
            (Type::Int, Type::Double) => true, // numeric coercion
            (Type::Null, Type::Nullable(_)) => true,
            (Type::Undefined, Type::Nullable(_)) => true,
            (ty, Type::Nullable(inner)) => ty.is_assignable_to(inner),
            (ty, Type::Union(types)) => types.iter().any(|t| ty.is_assignable_to(t)),
            // Function structural subtyping (naive)
            (Type::Function(s_args, s_ret), Type::Function(t_args, t_ret)) => {
                if s_args.len() != t_args.len() {
                    return false;
                }
                let args_match = s_args.iter().zip(t_args.iter()).all(|(s_arg, t_arg)| t_arg.is_assignable_to(s_arg));
                let ret_match = s_ret.is_assignable_to(t_ret);
                args_match && ret_match
            }
            // Object structural subtyping (duck typing)
            (Type::Object(s_props), Type::Object(t_props)) => {
                // For every property in the target type, the source type must have it and it must be assignable.
                t_props.iter().all(|(t_name, t_type)| {
                    s_props.iter()
                        .find(|(s_name, _)| s_name == t_name)
                        .map(|(_, s_type)| s_type.is_assignable_to(t_type))
                        .unwrap_or(false)
                })
            }
            (Type::Array(s_inner), Type::Array(t_inner)) => s_inner.is_assignable_to(t_inner),
            (Type::Tuple(s_types), Type::Tuple(t_types)) => {
                if s_types.len() != t_types.len() {
                    return false;
                }
                s_types.iter().zip(t_types.iter()).all(|(s, t)| s.is_assignable_to(t))
            }
            // Array/Tuple subtyping
            (Type::Tuple(s_types), Type::Array(t_inner)) => s_types.iter().all(|s| s.is_assignable_to(t_inner)),
            // Generic subtyping (exact match for now)
            (Type::Generic(s_name, s_args), Type::Generic(t_name, t_args)) => {
                if s_name != t_name || s_args.len() != t_args.len() {
                    return false;
                }
                s_args.iter().zip(t_args.iter()).all(|(s_arg, t_arg)| s_arg.is_assignable_to(t_arg))
            }
            _ => false,
        }
    }
}

// ── Operators ────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Not,      // !x
    Neg,      // -x
    Pos,      // +x (coerce to number)
    TypeOf,   // typeof x
    VoidOp,   // void x
    Delete,   // delete x
    BitNot,   // ~x
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Exp,
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    // Logical
    And, Or,
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr, UShr,
    // String concatenation (resolved during type checking)
    StrConcat,
    // Nullish coalescing
    NullishCoalesce,
    // instanceof
    InstanceOf,
    // `"key" in obj` — prototype-aware property check
    In,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AssignOp {
    Assign,         // =
    AddAssign,      // +=
    SubAssign,      // -=
    MulAssign,      // *=
    DivAssign,      // /=
    ModAssign,      // %=
    ExpAssign,      // **=
    BitAndAssign,   // &=
    BitOrAssign,    // |=
    BitXorAssign,   // ^=
    ShlAssign,      // <<=
    ShrAssign,      // >>=
    UShrAssign,     // >>>=
    LogAndAssign,   // &&= (ES2021)
    LogOrAssign,    // ||= (ES2021)
    NullishAssign,  // ??= (ES2021)
}

// ── Expressions ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum HirExpr {
    // Literals
    IntLit(i64),
    DoubleLit(f64),
    StringLit(String),
    BoolLit(bool),
    NullLit,
    UndefinedLit,
    BigIntLit(String),
    RegExpLit(String, String),

    // Variables & identifiers
    Var(String, Type),
    /// `this` keyword — loads slot 0 in instance methods
    This(Type),

    // Operations
    BinOp(BinOp, Box<HirExpr>, Box<HirExpr>, Type),
    UnaryOp(UnaryOp, Box<HirExpr>, Type),
    /// Ternary: `cond ? then_expr : else_expr`
    Ternary(Box<HirExpr>, Box<HirExpr>, Box<HirExpr>, Type),

    // Invocations
    Call(String, Vec<HirExpr>, Type),
    MethodCall(Box<HirExpr>, String, Vec<HirExpr>, Type),
    /// `super.method(args)` — uses invokespecial
    SuperCall(String, Vec<HirExpr>, Type),
    New(String, Vec<HirExpr>, Type),

    // Field access
    FieldGet(Box<HirExpr>, String, Type),
    FieldSet(Box<HirExpr>, String, Box<HirExpr>, Type),
    /// Computed property access: `obj[expr]` — uses Map.get or Array.get
    IndexGet(Box<HirExpr>, Box<HirExpr>, Type),
    /// Computed property set: `obj[expr] = val`
    IndexSet(Box<HirExpr>, Box<HirExpr>, Box<HirExpr>, Type),

    // Optional chaining: `obj?.prop` — null check before access
    OptionalChain(Box<HirExpr>, String, Type),
    OptionalCall(Box<HirExpr>, Vec<HirExpr>, Type),
    DynamicCall(Box<HirExpr>, Vec<HirExpr>, Type),

    // Collection literals
    ArrayLit(Vec<HirExpr>, Type),
    ObjectLit(Vec<ObjectProp>, Type),

    // Object operations
    /// `delete obj.prop` — returns bool
    DeleteProp(Box<HirExpr>, String),
    /// `Object.keys(obj)`, `Object.assign(a,b)`, etc.
    ObjectMethod(String, Vec<HirExpr>, Type),

    // Type operations
    Cast(Box<HirExpr>, Type),
    /// `x instanceof ClassName`
    InstanceOf(Box<HirExpr>, String),

    // Template literal: `hello ${name}, you are ${age}`
    TemplateLit(Vec<HirExpr>),

    // Arrow/lambda function
    Arrow(Vec<FnArg>, Type, Vec<HirStmt>, FnModifiers),

    // Spread: `...arr` — expanded during lowering
    Spread(Box<HirExpr>),

    // Await: `await promise` — lowered to CompletableFuture.get()
    Await(Box<HirExpr>, Type),

    // Sequence operator: `(a, b, c)` — evaluates all and returns last
    Seq(Vec<HirExpr>, Type),

    // Spawns a virtual thread for async bodies
    VirtualThreadSpawn(Box<HirExpr>),

    // Prefix/postfix increment/decrement
    PreIncrement(String),
    PreDecrement(String),
    PostIncrement(String),
    PostDecrement(String),

    // Logical assignment patterns
    LogicalAnd(Box<HirExpr>, Box<HirExpr>, Type),
    LogicalOr(Box<HirExpr>, Box<HirExpr>, Type),

    // Array operations — bridging JS Array to java.util.ArrayList
    /// `arr.length` — ArrayList.size()
    ArrayLen(Box<HirExpr>),
    /// `arr.push(x)`, `arr.pop()`, etc. — JS array methods
    ArrayMethod(Box<HirExpr>, String, Vec<HirExpr>, Type),
    /// Dynamic import("module") — CompletableFuture resolving to Class
    DynamicImport(Box<HirExpr>),
    /// Tagged Template Literal: tag`hello ${name}`
    TaggedTemplate(Box<HirExpr>, Vec<HirExpr>, Vec<HirExpr>),
    /// Yield Expression: yield val or yield* val
    Yield(Option<Box<HirExpr>>, bool),
    /// Special generator sentinel representing completion
    GeneratorSentinel,
}

/// Object literal property kinds — supports all TypeScript object literal forms.
#[derive(Clone, Debug)]
pub enum ObjectProp {
    /// `key: value`
    KeyValue(String, HirExpr),
    /// `[expr]: value` — computed property name
    Computed(HirExpr, HirExpr),
    /// `{ method() { ... } }` — method shorthand
    Method(String, Vec<FnArg>, Type, Vec<HirStmt>),
    /// `get name() { ... }` — holds a compiled Arrow expression
    Getter(String, HirExpr),
    /// `set name(param) { ... }` — holds a compiled Arrow expression
    Setter(String, HirExpr),
    /// `{ ...other }` — spread into object
    Spread(HirExpr),
}

impl HirExpr {
    pub fn get_type(&self) -> Type {
        match self {
            HirExpr::IntLit(_) => Type::Int,
            HirExpr::DoubleLit(_) => Type::Double,
            HirExpr::StringLit(_) => Type::StringTy,
            HirExpr::BoolLit(_) => Type::Bool,
            HirExpr::NullLit => Type::Null,
            HirExpr::UndefinedLit => Type::Undefined,
            HirExpr::BigIntLit(_) => Type::BigInt,
            HirExpr::RegExpLit(_, _) => Type::RegExp,
            HirExpr::Var(_, t) => t.clone(),
            HirExpr::This(t) => t.clone(),
            HirExpr::BinOp(_, _, _, t) => t.clone(),
            HirExpr::UnaryOp(_, _, t) => t.clone(),
            HirExpr::Ternary(_, _, _, t) => t.clone(),
            HirExpr::Call(_, _, t) => t.clone(),
            HirExpr::MethodCall(_, _, _, t) => t.clone(),
            HirExpr::SuperCall(_, _, t) => t.clone(),
            HirExpr::New(_, _, t) => t.clone(),
            HirExpr::FieldGet(_, _, t) => t.clone(),
            HirExpr::FieldSet(_, _, _, t) => t.clone(),
            HirExpr::IndexGet(_, _, t) => t.clone(),
            HirExpr::IndexSet(_, _, _, t) => t.clone(),
            HirExpr::OptionalChain(_, _, t) => t.clone(),
            HirExpr::OptionalCall(_, _, t) => t.clone(),
            HirExpr::DynamicCall(_, _, t) => t.clone(),
            HirExpr::ArrayLit(_, t) => t.clone(),
            HirExpr::ObjectLit(_, t) => t.clone(),
            HirExpr::DeleteProp(_, _) => Type::Bool,
            HirExpr::ObjectMethod(_, _, t) => t.clone(),
            HirExpr::Cast(_, t) => t.clone(),
            HirExpr::InstanceOf(_, _) => Type::Bool,
            HirExpr::TemplateLit(_) => Type::StringTy,
            HirExpr::Arrow(args, ret, _, _) => Type::Function(
                args.iter().map(|arg| arg.ty.clone()).collect(),
                Box::new(ret.clone()),
            ),
            HirExpr::Spread(inner) => inner.get_type(),
            HirExpr::Await(_, t) => t.clone(),
            HirExpr::Seq(_, t) => t.clone(),
            HirExpr::VirtualThreadSpawn(_) => Type::Class("java.util.concurrent.CompletableFuture".to_string()),
            HirExpr::PreIncrement(_) | HirExpr::PostIncrement(_) => Type::Double,
            HirExpr::PreDecrement(_) | HirExpr::PostDecrement(_) => Type::Double,
            HirExpr::LogicalAnd(_, _, t) => t.clone(),
            HirExpr::LogicalOr(_, _, t) => t.clone(),
            HirExpr::ArrayLen(_) => Type::Double,
            HirExpr::ArrayMethod(_, _, _, t) => t.clone(),
            HirExpr::DynamicImport(_) => Type::Class("java.util.concurrent.CompletableFuture".to_string()),
            HirExpr::TaggedTemplate(_, _, _) => Type::Any,
            HirExpr::Yield(_, _) => Type::Any,
            HirExpr::GeneratorSentinel => Type::Any,
        }
    }
}

// ── Statements ───────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum HirStmt {
    // Variable binding & assignment
    Let(String, Type, HirExpr),
    Const(String, Type, HirExpr),
    Assign(String, HirExpr),
    /// Compound assignment: `x += 1`
    CompoundAssign(String, AssignOp, HirExpr),
    /// Field assignment: `obj.field = expr`
    FieldAssign(HirExpr, String, HirExpr),

    // Control flow
    Return(Option<HirExpr>),
    If(HirExpr, Vec<HirStmt>, Vec<HirStmt>),
    While(HirExpr, Vec<HirStmt>),
    DoWhile(Vec<HirStmt>, HirExpr),
    /// `for (const item of iterable) { body }` — uses Iterator protocol
    ForOf(String, Type, HirExpr, Vec<HirStmt>),
    /// `for await (const item of iterable) { body }` — awaits each item
    ForAwaitOf(String, Type, HirExpr, Vec<HirStmt>),
    /// `for (const key in obj) { body }` — uses keySet() iteration
    ForIn(String, HirExpr, Vec<HirStmt>),
    Switch(HirExpr, Vec<SwitchCase>),
    Break(Option<String>),
    Continue(Option<String>),
    /// Labeled statement (for break/continue targets)
    Labeled(String, Box<HirStmt>),

    // Error handling
    TryCatch(Vec<HirStmt>, Option<String>, Vec<HirStmt>, Vec<HirStmt>),
    Throw(HirExpr),

    // Expressions as statements
    Expr(HirExpr),

    // Declarations
    FnDecl(String, Vec<FnArg>, Type, Vec<HirStmt>, FnModifiers),
    ClassDecl(String, ClassDef),
    InterfaceDecl(String, InterfaceDef),

    // Module system
    /// `export function foo() {}` — marks the function as public
    Export(Box<HirStmt>),
    /// `import { foo } from "./module"` — resolved to invokestatic on target class
    Import(Vec<ImportBinding>, String),

    // Destructuring
    /// `const { a, b: alias, c = default, ...rest } = expr`
    /// Fields: (prop_name, local_alias, default_value), rest_name, source
    DestructureObject(Vec<(String, Option<String>, Option<HirExpr>)>, Option<String>, HirExpr),
    /// `const [a, b, c = default, ...rest] = expr`
    /// Slots: (local_name or None to skip, default_value), rest_name, source
    DestructureArray(Vec<(Option<String>, Option<HirExpr>)>, Option<String>, HirExpr),
}

#[derive(Clone, Debug, Default)]
pub struct FnModifiers {
    pub is_async: bool,
    pub is_generator: bool,
    pub is_export: bool,
}

#[derive(Clone, Debug)]
pub struct FnArg {
    pub name: String,
    pub ty: Type,
    pub is_rest: bool,
}

// ── Class & Interface Definitions ────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ClassDef {
    /// Superclass name (e.g. "Animal"), or None for Object
    pub extends: Option<String>,
    /// Implemented interfaces
    pub implements: Vec<String>,
    /// Whether this class is abstract
    pub is_abstract: bool,
    /// Class members
    pub members: Vec<ClassMember>,
}

#[derive(Clone, Debug)]
pub struct InterfaceDef {
    /// Extended interfaces
    pub extends: Vec<String>,
    /// Interface method signatures (no bodies)
    pub methods: Vec<InterfaceMethod>,
    /// Interface fields (constants)
    pub fields: Vec<(String, Type)>,
}

#[derive(Clone, Debug)]
pub struct InterfaceMethod {
    pub name: String,
    pub args: Vec<FnArg>,
    pub return_type: Type,
}

#[derive(Clone, Debug)]
pub enum ClassMember {
    Field(String, Type, Option<HirExpr>, FieldModifiers),
    Method(String, Vec<(String, Type)>, Type, Vec<HirStmt>, MethodModifiers),
    Constructor(Vec<(String, Type)>, Vec<HirStmt>),
    /// Static initializer block
    StaticInit(Vec<HirStmt>),
    /// Getter: `get name() { ... }`
    Getter(String, Type, Vec<HirStmt>, bool),
    /// Setter: `set name(val) { ... }`
    Setter(String, String, Type, Vec<HirStmt>, bool),
}

#[derive(Clone, Debug, Default)]
pub struct FieldModifiers {
    pub is_static: bool,
    pub is_readonly: bool,
    pub is_private: bool,
    pub is_protected: bool,
    pub is_override: bool,
    pub is_abstract: bool,
}

#[derive(Clone, Debug, Default)]
pub struct MethodModifiers {
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_private: bool,
    pub is_protected: bool,
    pub is_async: bool,
    pub is_override: bool,
}

// ── Supporting Structures ────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SwitchCase {
    pub test: Option<HirExpr>,
    pub cons: Vec<HirStmt>,
}

#[derive(Clone, Debug)]
pub struct ImportBinding {
    /// The name imported (e.g. `foo` in `import { foo }`)
    pub name: String,
    /// Optional alias (e.g. `bar` in `import { foo as bar }`)
    pub alias: Option<String>,
}

// ── JVM Descriptor Helpers ───────────────────────────────────────────

/// Build a JVM method descriptor from parameter types and return type.
/// E.g. `(DLjava/lang/String;)Z` for `(double, String) -> boolean`
pub fn build_method_descriptor(params: &[Type], ret: &Type) -> String {
    let param_descs: String = params.iter().map(|t| t.to_jvm_descriptor()).collect();
    format!("({}){}", param_descs, ret.to_jvm_descriptor())
}
