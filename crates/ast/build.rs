use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use heck::ToSnakeCase;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use ungrammar::Grammar;
use ungrammar::Rule;

enum NodeKind {
    Node(String, Vec<Field>),
    Delegate(String, Vec<String>),
}

#[derive(Debug)]
struct Field {
    name: String,
    cardi: Cardinality,
    kind: FieldKind,
}

#[derive(Clone, Debug)]
enum FieldKind {
    Node,
    Token,
}

#[derive(Debug)]
enum Cardinality {
    One,
    Many,
}

fn main() {
    let grm_text = fs::read_to_string("ast.ungram").map(|s| s.replace("\r\n", "\n")).unwrap();
    let grm = Grammar::from_str(&grm_text).unwrap();

    let mut file = quote! {
        use rowan::ast::AstNode;
        use rowan::Language;
        use rowan::NodeOrToken;
        use syntax::SyntaxKind;
        use syntax::BbLang;

        use crate::SyntaxNode;
        use crate::SyntaxToken;
    };

    file.extend(
        grm.iter()
            .map(|node| impl_node(layout_rule(&grm, &grm[node].rule, grm[node].name.to_owned()))),
    );

    let mut out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_path.push("/ast_impl.rs");
    fs::write(out_path, file.to_string()).unwrap();
}

fn impl_node(node: NodeKind) -> TokenStream {
    match node {
        NodeKind::Node(node_name, fields) => {
            let node_name = format_ident!("{}", node_name);

            let mut imp = quote! {
                #[derive(Debug, Clone)]
                pub struct #node_name(SyntaxNode);
                impl AstNode for #node_name {
                    type Language = BbLang;
                    fn can_cast(kind: <Self::Language as Language>::Kind) -> bool { matches!(kind, SyntaxKind::#node_name) }
                    fn cast(syntax: SyntaxNode) -> Option<Self> { if Self::can_cast(syntax.kind()) { Some(Self(syntax)) } else { None } }
                    fn syntax(&self) -> &SyntaxNode { &self.0 }
                }
            };

            let field_imps: Vec<TokenStream> = fields
                .into_iter()
                .map(|Field { name, cardi, kind }| {
                    let mut func_stem = name.to_snake_case();
                    let ty_ident = format_ident!("{}", name);

                    let (func_ident, imp) = match cardi {
                        Cardinality::One => (
                            format_ident!("{}", &func_stem),
                            match kind {
                                FieldKind::Node => quote! { Option<#ty_ident> { self.0.children().find_map(#ty_ident::cast) } } ,
                                FieldKind::Token => quote! { Option<SyntaxToken> { self.0.children_with_tokens().find_map(|c| match c { NodeOrToken::Token(c) if matches!(c.kind(), SyntaxKind::#ty_ident) => Some(c), _ => None, }) } },
                            },
                        ),
                        Cardinality::Many => {
                            pluralize(&mut func_stem);

                            (
                                format_ident!("{}", func_stem),
                                match kind {
                                    FieldKind::Node => quote! { impl Iterator<Item = #ty_ident> { self.0.children().filter_map(#ty_ident::cast) } } ,
                                    FieldKind::Token => quote! { impl Iterator<Item = SyntaxToken> { self.0.children_with_tokens().filter_map(|c| match c { NodeOrToken::Token(c) if matches!(c.kind(), SyntaxKind::#ty_ident) => Some(c), _ => None, }) } },
                                },
                            )
                        },
                    };

                    quote! { pub fn #func_ident(&self) -> #imp }
                })
                .collect();

            imp.extend(quote! { impl #node_name { #(#field_imps)* }});

            imp
        }
        NodeKind::Delegate(node_name, variants) => {
            let node_name = format_ident!("{}", node_name);
            let (variants, try_intos): (Vec<Ident>, Vec<Ident>) = variants
                .into_iter()
                .map(|name| {
                    (format_ident!("{}", name), format_ident!("try_into_{}", name.to_snake_case()))
                })
                .unzip();

            quote! {
                #[derive(Debug, Clone)]
                pub enum #node_name { #(#variants(#variants)),* }
                impl AstNode for #node_name {
                    type Language = BbLang;
                    fn can_cast(kind: <Self::Language as Language>::Kind) -> bool { matches!(kind, #(SyntaxKind::#variants)|*) }
                    fn cast(syntax: SyntaxNode) -> Option<Self> { match syntax.kind() { #(SyntaxKind::#variants => Some(Self::#variants(#variants::cast(syntax).unwrap())),)* _ => None } }
                    fn syntax(&self) -> &SyntaxNode { match self { #(Self::#variants(node) => node.syntax()),* } }
                }
                impl #node_name {
                    #(pub fn #try_intos(self) -> Result<#variants, Self> { match self { Self::#variants(n) => Ok(n), _ => Err(self), } })*
                }
            }
        }
    }
}

fn layout_rule(grm: &Grammar, rule: &Rule, name: String) -> NodeKind {
    return match rule {
        Rule::Node(node) => NodeKind::Node(
            name,
            vec![Field {
                name: grm[*node].name.clone(),
                cardi: Cardinality::One,
                kind: FieldKind::Node,
            }],
        ),
        Rule::Token(tok) => NodeKind::Node(
            name,
            vec![Field {
                name: token_to_syntaxkind(&grm[*tok].name),
                cardi: Cardinality::One,
                kind: FieldKind::Token,
            }],
        ),
        Rule::Alt(alts) => NodeKind::Delegate(
            name,
            alts.iter()
                .map(|alt| match alt {
                    Rule::Node(node) => grm[*node].name.clone(),
                    Rule::Labeled { .. }
                    | Rule::Token(_)
                    | Rule::Seq(_)
                    | Rule::Alt(_)
                    | Rule::Opt(_)
                    | Rule::Rep(_) => panic!("Alternatives (with '|') must be nodes: {alt:?}"),
                })
                .collect(),
        ),
        Rule::Opt(inner) => layout_rule(grm, inner, name),
        Rule::Seq(inner) => {
            let fields: Vec<Field> = inner
                .iter()
                .map(|rule| {
                    let (name, kind) =
                        name_atom(grm, rule).expect("Ungrammar alts must be top-level");
                    Field { name, kind, cardi: Cardinality::One }
                })
                .collect();

            // fields must be unique
            assert_eq!(
                fields
                    .iter()
                    .map(|Field { name, .. }| name.as_ref())
                    .collect::<HashSet<&str>>()
                    .len(),
                fields.len()
            );

            NodeKind::Node(name, fields)
        }
        Rule::Rep(inner) => NodeKind::Node(
            name,
            vec![{
                let (name, kind) = name_atom(grm, inner).unwrap();
                Field { name, kind, cardi: Cardinality::Many }
            }],
        ),
        Rule::Labeled { .. } => {
            unreachable!("this syntax generator does not support labels: {rule:?}")
        }
    };
}

fn name_atom(grm: &Grammar, rule: &Rule) -> Option<(String, FieldKind)> {
    match rule {
        Rule::Node(node) => Some((grm[*node].name.clone(), FieldKind::Node)),
        Rule::Token(tok) => Some((token_to_syntaxkind(&grm[*tok].name), FieldKind::Token)),
        Rule::Opt(inner) => name_atom(grm, inner), // optionals of atoms are fine
        Rule::Labeled { .. } | Rule::Seq(_) | Rule::Alt(_) | Rule::Rep(_) => None,
    }
}

fn token_to_syntaxkind(tok: &str) -> String {
    match tok {
        "todo" => "DUMMY",
        _ => panic!("unexpected token: {tok:?}:"),
    }
    .to_owned()
}

fn pluralize(s: &mut String) {
    s.push('s');
}
