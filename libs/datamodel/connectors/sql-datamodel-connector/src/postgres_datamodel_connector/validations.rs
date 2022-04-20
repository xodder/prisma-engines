use datamodel_connector::{
    parser_database::{walkers::IndexWalker, IndexAlgorithm, OperatorClass},
    walker_ext_traits::*,
    Connector, DatamodelError, Diagnostics,
};
use native_types::PostgresType;

pub(super) fn compatible_native_types(index: IndexWalker<'_>, connector: &dyn Connector, errors: &mut Diagnostics) {
    for field in index.fields() {
        if let Some(native_type) = field.native_type_instance(connector) {
            let span = field.ast_field().span;
            let r#type: PostgresType = serde_json::from_value(native_type.serialized_native_type.clone()).unwrap();
            let error = connector.native_instance_error(&native_type);

            if r#type == PostgresType::Xml {
                if index.is_unique() {
                    errors.push_error(error.new_incompatible_native_type_with_unique("", span))
                } else {
                    errors.push_error(error.new_incompatible_native_type_with_index("", span))
                };

                break;
            }
        }
    }
}

/// Validating the correct usage of GiST/GIN/SP-GiST and BRIN indices.
pub(super) fn generalized_index_validations(
    index: IndexWalker<'_>,
    connector: &dyn Connector,
    errors: &mut Diagnostics,
) {
    let algo = index.algorithm().unwrap_or(IndexAlgorithm::BTree);

    for field in index.scalar_field_attributes() {
        let native_type = field
            .as_index_field()
            .native_type_instance(connector)
            .map(|t| serde_json::from_value(t.serialized_native_type).unwrap());

        let opclass = field.operator_class().and_then(|c| c.get().left());

        let attr = match index.ast_attribute() {
            Some(attr) => attr,
            _ => continue,
        };

        match (opclass, algo) {
            // valid gist
            (Some(OperatorClass::InetOps), IndexAlgorithm::Gist) => (),

            // invalid
            (Some(opclass), _) => {
                let msg =
                    format!("The given operator class `{opclass}` is not supported with the `{algo}` index type.");

                errors.push_error(DatamodelError::new_attribute_validation_error(
                    &msg, "@index", attr.span,
                ));
            }

            // others
            _ => (),
        }

        match (&native_type, opclass) {
            (Some(PostgresType::Inet), Some(OperatorClass::InetOps)) => (),

            // error
            (Some(native_type), Some(opclass)) => {
                let name = field.as_index_field().name();

                let msg = format!(
                    "The given operator class `{opclass}` does not support `{name}` field's native type `{native_type}`."
                );

                errors.push_error(DatamodelError::new_attribute_validation_error(
                    &msg, "@index", attr.span,
                ));
            }
            (Some(native_type), None) => {
                let msg = format!("The {algo} index field type `{native_type}` has no default operator class.");

                errors.push_error(DatamodelError::new_attribute_validation_error(
                    &msg, "@index", attr.span,
                ));
            }
            (None, Some(opclass)) => {
                let name = field.as_index_field().name();
                let msg =
                    format!("The given operator class `{opclass}` expects the field `{name}` to define a native type.");

                errors.push_error(DatamodelError::new_attribute_validation_error(
                    &msg, "@index", attr.span,
                ));
            }
            _ => (),
        }
    }
}
