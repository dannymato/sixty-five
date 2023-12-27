use proc_macro2::{self, Ident};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Field, Fields, Type, Variant};

#[proc_macro_derive(OpcodeDecoder)]
pub fn derive_decoder(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let impl_gen = generate_code(&input);
    impl_gen.into()
}

fn generate_code(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;
    let enum_data = pull_enum(ast);
    let arms = enum_data
        .variants
        .iter()
        .map(|var| generate_arm(ident, var));
    let tokens = quote! {
        use crate::sixty_five::{memory_bus::MemoryBus, cpu::Cpu};
        impl OpcodeDecoder for #ident {
            fn decode_opcode(cpu: &mut Cpu, memory: &MemoryBus) -> Opcode {
                let opcode = cpu.fetch_byte(&memory);
                match opcode {
                    #(#arms)*
                    // TODO: This should not panic
                    _ => panic!("Unknown opcode")
                }
            }
        }
    };

    tokens
}

fn pull_enum(derive: &DeriveInput) -> &DataEnum {
    match &derive.data {
        Data::Enum(e) => e,
        _ => panic!("OpcodeDecoder only supports enums"),
    }
}

fn generate_arm(enum_ident: &Ident, var: &Variant) -> proc_macro2::TokenStream {
    let (_, opcode_value) = &var
        .discriminant
        .as_ref()
        .expect("Variant must have a value assigned");
    let ident = &var.ident;
    let params = &var.fields;
    match params {
        Fields::Unit => quote! {
            #opcode_value => #enum_ident::#ident,
        },
        Fields::Named(_) => panic!("Named fields are unsupported in enums"),
        Fields::Unnamed(fields) => {
            let idents: Vec<Ident> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("param_{}", i))
                .collect();

            let parts = fields.unnamed.iter().zip(&idents).map(|(field, ident)| {
                let param_type = type_name_from_field(field);
                match param_type.as_str() {
                    "Word" => quote! {
                        let #ident = cpu.fetch_word(&memory);
                    },
                    "Byte" => quote! {
                        let #ident = cpu.fetch_byte(&memory);
                    },
                    _ => panic!("Only Byte and Word are supported"),
                }
            });

            quote! {
                #opcode_value => {
                    #(#parts)*
                    #enum_ident::#ident(#(#idents),*)
                },
            }
        }
    }
}

fn type_name_from_field(field: &Field) -> String {
    match &field.ty {
        Type::Path(path) => path.path.segments.last().unwrap().ident.to_string(),
        _ => panic!("Only supporting regular types for enum params"),
    }
}
