use pliantdb_core::{
    schema::{InvalidNameError, Schema, SchemaName, Schematic},
    Error,
};

pub mod database;
pub mod encryption_key;

#[derive(Debug)]
pub struct Admin;

impl Schema for Admin {
    fn schema_name() -> Result<SchemaName, InvalidNameError> {
        SchemaName::new("khonsulabs", "pliantdb-admin")
    }

    fn define_collections(schema: &mut Schematic) -> Result<(), Error> {
        schema.define_collection::<database::Database>()?;
        schema.define_collection::<encryption_key::EncryptionKeyVersion>()?;

        Ok(())
    }
}
