use quote::__private::TokenStream;
use quote::*;
use regex::Regex;

#[derive(Clone, Debug)]
pub enum IceType {
    VoidType,
    BoolType,
    ByteType,
    ShortType,
    IntType,
    LongType,
    FloatType,
    DoubleType,
    StringType,    
    SequenceType(Box<IceType>),
    DictType(Box<IceType>, Box<IceType>),
    Optional(Box<IceType>, u8),
    CustomType(String)
}

impl IceType {
    pub fn from(text: &str) -> Result<IceType, Box<dyn std::error::Error>> {
        let type_re = Regex::new(
            r#"(?x)
            (void) |
            (bool) |
            (byte) |
            (short) |
            (int) |
            (long) |
            (float) |
            (double) |
            (string) |
            (sequence)<(.+)> |
            (dictionary)<(.+),\s*(.+)> |
            "#
        )?; 

        let captures = type_re.captures(text.trim()).map(|captures| {
            captures
                .iter() // All the captured groups
                .skip(1) // Skipping the complete match
                .flat_map(|c| c) // Ignoring all empty optional matches
                .map(|c| c.as_str()) // Grab the original strings
                .collect::<Vec<_>>() // Create a vector
        });

        match captures.as_ref().map(|c| c.as_slice()) {
            Some(["void"]) => Ok(IceType::VoidType),
            Some(["bool"]) => Ok(IceType::BoolType),
            Some(["byte"]) => Ok(IceType::ByteType),
            Some(["short"]) => Ok(IceType::ShortType),
            Some(["int"]) => Ok(IceType::IntType),
            Some(["long"]) => Ok(IceType::LongType),
            Some(["float"]) => Ok(IceType::FloatType),
            Some(["double"]) => Ok(IceType::DoubleType),
            Some(["string"]) => Ok(IceType::StringType),
            Some(["sequence", x]) => {
                Ok(IceType::SequenceType(Box::new(IceType::from(x)?)))
            },
            Some(["dictionary", x, y]) => {
                Ok(IceType::DictType(Box::new(IceType::from(x)?), Box::new(IceType::from(y)?)))
            },
            _ => Ok(IceType::CustomType(text.trim().to_string()))
        }
    }

    pub fn rust_type(&self) -> String {
        match self {
            IceType::VoidType => String::from("()"),
            IceType::BoolType => String::from("bool"),
            IceType::ByteType => String::from("u8"),
            IceType::ShortType => String::from("i16"),
            IceType::IntType => String::from("i32"),
            IceType::LongType => String::from("i64"),
            IceType::FloatType => String::from("f32"),
            IceType::DoubleType => String::from("f64"),
            IceType::StringType => String::from("String"),
            IceType::SequenceType(type_name) => format!("Vec<{}>", type_name.rust_type()),
            IceType::DictType(key_type, value_type) => format!("HashMap<{}, {}>", key_type.rust_type(), value_type.rust_type()),
            IceType::Optional(type_name, _) => format!("Option<{}>", type_name.rust_type()),
            IceType::CustomType(type_name) => format!("{}", type_name),
        }
    }

    pub fn token_from(&self) -> TokenStream {
        match self {
            IceType::Optional(type_name, _) => {
                let sub_type = type_name.token();
                quote!{ Option::<#sub_type> }
            }
            IceType::SequenceType(type_name) => {
                let sub_type = type_name.token();
                quote!{ Vec::<#sub_type> }
            }
            _ => self.token(),
        }
    }

    pub fn token(&self) -> TokenStream {
        match self {
            IceType::VoidType => quote! { () },
            IceType::BoolType => quote! { bool },
            IceType::ByteType => quote! { u8 },
            IceType::ShortType => quote! { i16 },
            IceType::IntType => quote! { i32 },
            IceType::LongType => quote! { i64 },
            IceType::FloatType => quote! { f32 },
            IceType::DoubleType => quote! { f64 },
            IceType::StringType => quote! { String },
            IceType::SequenceType(type_name) => {
                let sub_type = type_name.token();
                quote!{ Vec<#sub_type> }
            },
            IceType::DictType(key_type, value_type) => {
                let key = key_type.token();
                let value = value_type.token();
                quote!{ HashMap<#key, #value> }
            },
            IceType::Optional(type_name, _) => {
                let sub_type = type_name.token();
                quote!{ Option<#sub_type> }
            },
            IceType::CustomType(type_name) => {
                let id = format_ident!("{}", type_name);
                quote!{ #id }
            },
        }
    }

    pub fn as_ref(&self) -> bool {
        match self {
            IceType::StringType |
            IceType::SequenceType(_) |
            IceType::DictType(_, _) |
            IceType::CustomType(_) => true,
            _ => false
        }
    }
}