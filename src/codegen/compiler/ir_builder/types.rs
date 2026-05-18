use swc_core::ecma::ast::*;
use crate::codegen::compiler::ir::*;
use super::IrBuilder;

impl IrBuilder {
    pub(crate) fn extract_type(&self, type_ann: Option<&TsTypeAnn>) -> Type {
        if let Some(ann) = type_ann {
            match &*ann.type_ann {
                TsType::TsKeywordType(kw) => match kw.kind {
                    TsKeywordTypeKind::TsNumberKeyword => Type::Double,
                    TsKeywordTypeKind::TsStringKeyword => Type::StringTy,
                    TsKeywordTypeKind::TsBooleanKeyword => Type::Bool,
                    TsKeywordTypeKind::TsVoidKeyword => Type::Void,
                    TsKeywordTypeKind::TsAnyKeyword => Type::Any,
                    TsKeywordTypeKind::TsNullKeyword => Type::Null,
                    TsKeywordTypeKind::TsUndefinedKeyword => Type::Undefined,
                    TsKeywordTypeKind::TsUnknownKeyword => Type::Unknown,
                    TsKeywordTypeKind::TsBigIntKeyword => Type::BigInt,
                    TsKeywordTypeKind::TsSymbolKeyword => Type::Symbol,
                    _ => Type::Any,
                },
                TsType::TsTypeRef(type_ref) => {
                    if let TsEntityName::Ident(ident) = &type_ref.type_name {
                        let name = ident.sym.to_string();
                        if let Some(type_params) = &type_ref.type_params {
                            let args = type_params.params.iter().map(|p| self.extract_type(Some(&TsTypeAnn { span: Default::default(), type_ann: p.clone() }))).collect();
                            Type::Generic(name, args)
                        } else {
                            // First, try to resolve it as a custom type (interface/alias)
                            let resolved = self.env.resolve_type(&name);
                            if resolved != Type::Any {
                                resolved
                            } else {
                                Type::Class(name)
                            }
                        }
                    } else {
                        Type::Any
                    }
                }
                TsType::TsArrayType(arr) => {
                    let inner_type = self.extract_type(Some(&TsTypeAnn { span: Default::default(), type_ann: arr.elem_type.clone() }));
                    Type::Array(Box::new(inner_type))
                }
                TsType::TsTupleType(tup) => {
                    let mut types = Vec::new();
                    for elem in &tup.elem_types {
                        types.push(self.extract_type(Some(&TsTypeAnn { span: Default::default(), type_ann: elem.ty.clone() })));
                    }
                    Type::Tuple(types)
                }
                TsType::TsTypeLit(lit) => {
                    let mut props = Vec::new();
                    for member in &lit.members {
                        if let swc_core::ecma::ast::TsTypeElement::TsPropertySignature(sig) = member {
                            if let swc_core::ecma::ast::Expr::Ident(ident) = &*sig.key {
                                let name = ident.sym.to_string();
                                let ty = self.extract_type(sig.type_ann.as_deref());
                                props.push((name, ty));
                            }
                        }
                    }
                    Type::Object(props)
                }
                TsType::TsTypeOperator(op) => {
                    if op.op == TsTypeOperatorOp::ReadOnly || op.op == TsTypeOperatorOp::Unique {
                        // Unwrap and return the inner type
                        self.extract_type(Some(&TsTypeAnn { span: Default::default(), type_ann: op.type_ann.clone() }))
                    } else {
                        Type::Any
                    }
                }
                TsType::TsLitType(lit_type) => match &lit_type.lit {
                    TsLit::Number(_) => Type::Double,
                    TsLit::Str(_) => Type::StringTy,
                    TsLit::Bool(_) => Type::Bool,
                    TsLit::BigInt(_) => Type::BigInt,
                    TsLit::Tpl(_) => Type::StringTy,
                },
                TsType::TsTypePredicate(_) => Type::Bool,
                TsType::TsTypeQuery(query) => {
                    if let TsTypeQueryExpr::TsEntityName(TsEntityName::Ident(ident)) = &query.expr_name {
                        self.env.get_type(&ident.sym.to_string())
                    } else {
                        Type::Any
                    }
                }
                _ => Type::Any,
            }
        } else {
            Type::Any
        }
    }
}
