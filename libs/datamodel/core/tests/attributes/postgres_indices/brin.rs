use crate::{common::*, with_header, Provider};

#[test]
fn without_preview_feature() {
    let dml = indoc! {r#"
        model A {
          id Int  @id
          a  Int

          @@index([a(ops: raw("whatever_ops"))], type: Brin)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &[]);
    let error = datamodel::parse_schema(&schema).map(drop).unwrap_err();

    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@index": You must enable `extendedIndexes` preview feature to be able to define the index type.[0m
          [1;94m-->[0m  [4mschema.prisma:15[0m
        [1;94m   | [0m
        [1;94m14 | [0m
        [1;94m15 | [0m  @@index([a(ops: raw("whatever_ops"))], [1;91mtype: Brin[0m)
        [1;94m   | [0m
    "#]];

    expectation.assert_eq(&error)
}

#[test]
fn on_mysql() {
    let dml = indoc! {r#"
        model A {
          id Int  @id
          a  Int

          @@index([a(ops: raw("whatever_ops"))], type: Brin)
        }
    "#};

    let schema = with_header(dml, Provider::Mysql, &["extendedIndexes"]);
    let error = datamodel::parse_schema(&schema).map(drop).unwrap_err();

    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@index": The given index type is not supported with the current connector[0m
          [1;94m-->[0m  [4mschema.prisma:15[0m
        [1;94m   | [0m
        [1;94m14 | [0m
        [1;94m15 | [0m  @@index([a(ops: raw("whatever_ops"))], [1;91mtype: Brin[0m)
        [1;94m   | [0m
    "#]];

    expectation.assert_eq(&error)
}

#[test]
fn with_raw_unsupported() {
    let dml = indoc! {r#"
        model A {
          id Int                     @id
          a  Unsupported("tsvector")

          @@index([a(ops: raw("tsvector_ops"))], type: Brin)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let schema = parse(&schema);

    let mut field = IndexField::new_in_model("a");
    field.operator_class = Some(OperatorClass::raw("tsvector_ops"));

    schema.assert_has_model("A").assert_has_index(IndexDefinition {
        name: None,
        db_name: Some("A_a_idx".to_string()),
        fields: vec![field],
        tpe: IndexType::Normal,
        defined_on_field: false,
        algorithm: Some(IndexAlgorithm::Brin),
        clustered: None,
    });
}

#[test]
fn with_unsupported_no_ops() {
    let dml = indoc! {r#"
        model A {
          id Int                     @id
          a  Unsupported("tsvector")

          @@index([a], type: Brin)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let schema = parse(&schema);

    let field = IndexField::new_in_model("a");

    schema.assert_has_model("A").assert_has_index(IndexDefinition {
        name: None,
        db_name: Some("A_a_idx".to_string()),
        fields: vec![field],
        tpe: IndexType::Normal,
        defined_on_field: false,
        algorithm: Some(IndexAlgorithm::Brin),
        clustered: None,
    });
}
