use super::dmmf::get_query_schema;
use crate::dmmf::*;
use codegen::{Scope, Struct, Type};
use serial_test::serial;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;

// Tests in this file run serially because the function `get_query_schema` depends on setting an env var.

#[test]
#[serial]
fn client_generator_test() {
    let dm = r#"
        model Blog {
            blogId String @id
            name   String
        }

        model Post {
            postId   String  @id
            title    String
            subTitle String
            
            @@unique([title, subTitle])
        }
    "#;

    let (query_schema, datamodel) = get_query_schema(dm);

    let dmmf = crate::dmmf::render_dmmf(&datamodel, Arc::new(query_schema));

    dbg!(&dmmf);

    let inputs = &dmmf.schema.input_types;

    let mut scope = Scope::new();

    scope.raw("#![allow(non_snake_case, dead_code, unused_variables, non_camel_case_types)]");
    scope.raw("/// INPUT TYPES");
    for input in inputs.iter() {
        if input.is_one_of {
            let input_enum = scope.new_enum(&input.name).vis("pub");
            for field in input.fields.iter() {
                input_enum
                    .new_variant(&field.name.pascal_case())
                    .tuple(&field.input_type.typ);
            }
        } else {
            let input_struct = new_struct(&mut scope, &input.name);
            for field in input.fields.iter() {
                input_struct.field(&format!("pub {}", &field.name), map_type(&field.input_type));
            }
        }
    }

    scope.raw("/// OUTPUT TYPES");

    let non_model_types = vec!["Query", "Mutation"];
    let model_output_types: Vec<_> = dmmf
        .schema
        .output_types
        .iter()
        .filter(|ot| !non_model_types.contains(&ot.name.as_str()))
        .collect();

    for output in model_output_types.iter() {
        let output_struct = new_struct(&mut scope, &output.name);
        for field in output.fields.iter() {
            output_struct.field(&format!("pub {}", &field.name), map_type(&field.output_type));
        }
    }

    let query_type = dmmf.schema.output_types.iter().find(|ot| ot.name == "Query").unwrap();
    let mutation_type = dmmf
        .schema
        .output_types
        .iter()
        .find(|ot| ot.name == "Mutation")
        .unwrap();

    let _ = new_struct(&mut scope, "PrismaClient");
    let client_impl = scope.new_impl("PrismaClient");
    client_impl.new_fn("new").vis("pub").ret("PrismaClient").line("todo!()");

    for query_field in query_type.fields.iter() {
        let function = client_impl.new_fn(&query_field.name);
        function.arg_ref_self();
        for arg in query_field.args.iter() {
            let arg_name = if &arg.name == "where" { "where_" } else { &arg.name };

            function.arg(arg_name, &map_type(&arg.input_type)).vis("pub");
        }
        function.ret(&map_type(&query_field.output_type));
        function.line("todo!()");
    }

    for mutation_field in mutation_type.fields.iter() {
        let function = client_impl.new_fn(&mutation_field.name);
        function.arg_ref_self();
        for arg in mutation_field.args.iter() {
            let arg_name = if &arg.name == "where" { "where_" } else { &arg.name };

            function.arg(arg_name, &map_type(&arg.input_type)).vis("pub");
            function.ret(&map_type(&mutation_field.output_type));
        }
        function.line("todo!()");
    }

    for an_enum in dmmf.schema.enums.iter() {
        let the_enum = scope.new_enum(&an_enum.name).vis("pub");
        for variant in an_enum.values.iter() {
            the_enum.new_variant(&variant);
        }
    }

    write_to_disk(&scope.to_string());
}

fn map_type(dmmf_type: &DMMFTypeInfo) -> Type {
    let x = match dmmf_type.typ.as_str() {
        "Int" => "i64",
        x => x,
    };

    if dmmf_type.is_nullable {
        let mut tpe = Type::new("Option");
        tpe.generic(x);
        tpe
    } else if dmmf_type.is_list {
        let mut tpe = Type::new("Vec");
        tpe.generic(x);
        tpe
    } else {
        Type::new(x)
    }
}

fn new_struct<'a>(scope: &'a mut Scope, name: &str) -> &'a mut Struct {
    scope.new_struct(name).vis("pub")
}

fn write_to_disk(content: &str) {
    let path = "/Users/marcusboehm/R/github.com/prisma/rust-client-test/src/client.rs";
    let mut file = File::create(path).expect("creating file failed");
    file.write_all(content.as_bytes()).expect("writing to file failed");
}

pub trait NameNormalizer {
    fn camel_case(&self) -> String;

    fn pascal_case(&self) -> String;
}

impl NameNormalizer for String {
    fn camel_case(&self) -> String {
        let mut c = self.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        }
    }

    fn pascal_case(&self) -> String {
        let mut c = self.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}
