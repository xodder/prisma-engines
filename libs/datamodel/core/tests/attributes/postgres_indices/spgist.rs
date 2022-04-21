use crate::{common::*, with_header, Provider};

#[test]
fn without_preview_feature() {
    let dml = indoc! {r#"
        model A {
          id Int                    @id
          a  Unsupported("polygon")

          @@index([a(ops: raw("poly_ops"))], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &[]);
    let error = datamodel::parse_schema(&schema).map(drop).unwrap_err();

    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@index": You must enable `extendedIndexes` preview feature to be able to define the index type.[0m
          [1;94m-->[0m  [4mschema.prisma:15[0m
        [1;94m   | [0m
        [1;94m14 | [0m
        [1;94m15 | [0m  @@index([a(ops: raw("poly_ops"))], [1;91mtype: SpGist[0m)
        [1;94m   | [0m
    "#]];

    expectation.assert_eq(&error)
}

#[test]
fn on_mysql() {
    let dml = indoc! {r#"
        model A {
          id Int                    @id
          a  Unsupported("polygon")

          @@index([a(ops: raw("poly_ops"))], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Mysql, &["extendedIndexes"]);
    let error = datamodel::parse_schema(&schema).map(drop).unwrap_err();

    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@index": The given index type is not supported with the current connector[0m
          [1;94m-->[0m  [4mschema.prisma:15[0m
        [1;94m   | [0m
        [1;94m14 | [0m
        [1;94m15 | [0m  @@index([a(ops: raw("poly_ops"))], [1;91mtype: SpGist[0m)
        [1;94m   | [0m
    "#]];

    expectation.assert_eq(&error)
}

#[test]
fn with_raw_unsupported() {
    let dml = indoc! {r#"
        model A {
          id Int                    @id
          a  Unsupported("polygon")

          @@index([a(ops: raw("poly_ops"))], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let schema = parse(&schema);

    let mut field = IndexField::new_in_model("a");
    field.operator_class = Some(OperatorClass::raw("poly_ops"));

    schema.assert_has_model("A").assert_has_index(IndexDefinition {
        name: None,
        db_name: Some("A_a_idx".to_string()),
        fields: vec![field],
        tpe: IndexType::Normal,
        defined_on_field: false,
        algorithm: Some(IndexAlgorithm::SpGist),
        clustered: None,
    });
}

#[test]
fn with_unsupported_no_ops() {
    let dml = indoc! {r#"
        model A {
          id Int                    @id
          a  Unsupported("polygon")

          @@index([a], type: SpGist)
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
        algorithm: Some(IndexAlgorithm::SpGist),
        clustered: None,
    });
}

// NetworkOps

#[test]
fn no_ops_inet_native_type() {
    let dml = indoc! {r#"
        model A {
          id Int    @id
          a  String @test.Inet

          @@index([a], type: SpGist)
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
        algorithm: Some(IndexAlgorithm::SpGist),
        clustered: None,
    });
}

#[test]
fn inet_type_network_ops() {
    let dml = indoc! {r#"
        model A {
          id Int    @id
          a  String @test.Inet

          @@index([a(ops: NetworkOps)], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let schema = parse(&schema);

    let mut field = IndexField::new_in_model("a");
    field.operator_class = Some(OperatorClass::NetworkOps);

    schema.assert_has_model("A").assert_has_index(IndexDefinition {
        name: None,
        db_name: Some("A_a_idx".to_string()),
        fields: vec![field],
        tpe: IndexType::Normal,
        defined_on_field: false,
        algorithm: Some(IndexAlgorithm::SpGist),
        clustered: None,
    });
}

#[test]
fn no_native_type_network_ops() {
    let dml = indoc! {r#"
        model A {
          id Int    @id
          a  String

          @@index([a(ops: NetworkOps)], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let schema = parse(&schema);

    let mut field = IndexField::new_in_model("a");
    field.operator_class = Some(OperatorClass::NetworkOps);

    schema.assert_has_model("A").assert_has_index(IndexDefinition {
        name: None,
        db_name: Some("A_a_idx".to_string()),
        fields: vec![field],
        tpe: IndexType::Normal,
        defined_on_field: false,
        algorithm: Some(IndexAlgorithm::SpGist),
        clustered: None,
    });
}

#[test]
fn network_ops_with_wrong_prisma_type() {
    let dml = indoc! {r#"
        model A {
          id Int  @id
          a  Int

          @@index([a(ops: NetworkOps)], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let error = datamodel::parse_schema(&schema).map(drop).unwrap_err();

    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@@index": The given operator class `NetworkOps` expects the field `a` to define a valid native type.[0m
          [1;94m-->[0m  [4mschema.prisma:15[0m
        [1;94m   | [0m
        [1;94m14 | [0m
        [1;94m15 | [0m  @@[1;91mindex([a(ops: NetworkOps)], type: SpGist)[0m
        [1;94m   | [0m
    "#]];

    expectation.assert_eq(&error)
}

#[test]
fn network_ops_with_wrong_index_type() {
    let dml = indoc! {r#"
        model A {
          id Int    @id
          a  String @test.Inet

          @@index([a(ops: NetworkOps)], type: Gist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let error = datamodel::parse_schema(&schema).map(drop).unwrap_err();

    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@@index": The given operator class `NetworkOps` is not supported with the `Gist` index type.[0m
          [1;94m-->[0m  [4mschema.prisma:15[0m
        [1;94m   | [0m
        [1;94m14 | [0m
        [1;94m15 | [0m  @@[1;91mindex([a(ops: NetworkOps)], type: Gist)[0m
        [1;94m   | [0m
    "#]];

    expectation.assert_eq(&error)
}

// TextOps

#[test]
fn no_ops_text_native_type() {
    let dml = indoc! {r#"
        model A {
          id Int    @id
          a  String @test.Text

          @@index([a], type: SpGist)
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
        algorithm: Some(IndexAlgorithm::SpGist),
        clustered: None,
    });
}

#[test]
fn text_type_text_ops() {
    let dml = indoc! {r#"
        model A {
          id Int    @id
          a  String @test.Text

          @@index([a(ops: TextOps)], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let schema = parse(&schema);

    let mut field = IndexField::new_in_model("a");
    field.operator_class = Some(OperatorClass::TextOps);

    schema.assert_has_model("A").assert_has_index(IndexDefinition {
        name: None,
        db_name: Some("A_a_idx".to_string()),
        fields: vec![field],
        tpe: IndexType::Normal,
        defined_on_field: false,
        algorithm: Some(IndexAlgorithm::SpGist),
        clustered: None,
    });
}

#[test]
fn no_native_type_text_ops() {
    let dml = indoc! {r#"
        model A {
          id Int    @id
          a  String

          @@index([a(ops: TextOps)], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let schema = parse(&schema);

    let mut field = IndexField::new_in_model("a");
    field.operator_class = Some(OperatorClass::TextOps);

    schema.assert_has_model("A").assert_has_index(IndexDefinition {
        name: None,
        db_name: Some("A_a_idx".to_string()),
        fields: vec![field],
        tpe: IndexType::Normal,
        defined_on_field: false,
        algorithm: Some(IndexAlgorithm::SpGist),
        clustered: None,
    });
}

#[test]
fn text_ops_with_wrong_prisma_type() {
    let dml = indoc! {r#"
        model A {
          id Int  @id
          a  Int

          @@index([a(ops: TextOps)], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let error = datamodel::parse_schema(&schema).map(drop).unwrap_err();

    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@@index": The given operator class `TextOps` points to the field `a` that is not of String type.[0m
          [1;94m-->[0m  [4mschema.prisma:15[0m
        [1;94m   | [0m
        [1;94m14 | [0m
        [1;94m15 | [0m  @@[1;91mindex([a(ops: TextOps)], type: SpGist)[0m
        [1;94m   | [0m
    "#]];

    expectation.assert_eq(&error)
}

#[test]
fn no_ops_weird_type() {
    let dml = indoc! {r#"
        model A {
          id Int  @id
          a  Int

          @@index([a], type: SpGist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let error = datamodel::parse_schema(&schema).map(drop).unwrap_err();

    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@@index": The SpGist index type does not support the type of the field `a`.[0m
          [1;94m-->[0m  [4mschema.prisma:15[0m
        [1;94m   | [0m
        [1;94m14 | [0m
        [1;94m15 | [0m  @@[1;91mindex([a], type: SpGist)[0m
        [1;94m   | [0m
    "#]];

    expectation.assert_eq(&error)
}

#[test]
fn text_ops_with_wrong_index_type() {
    let dml = indoc! {r#"
        model A {
          id Int    @id
          a  String @test.Text

          @@index([a(ops: TextOps)], type: Gist)
        }
    "#};

    let schema = with_header(dml, Provider::Postgres, &["extendedIndexes"]);
    let error = datamodel::parse_schema(&schema).map(drop).unwrap_err();

    let expectation = expect![[r#"
        [1;91merror[0m: [1mError parsing attribute "@@index": The given operator class `TextOps` is not supported with the `Gist` index type.[0m
          [1;94m-->[0m  [4mschema.prisma:15[0m
        [1;94m   | [0m
        [1;94m14 | [0m
        [1;94m15 | [0m  @@[1;91mindex([a(ops: TextOps)], type: Gist)[0m
        [1;94m   | [0m
    "#]];

    expectation.assert_eq(&error)
}
