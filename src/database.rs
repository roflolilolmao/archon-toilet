use postgres_types::{Oid, Type};
use serde::Serialize;
use tokio_postgres::{Client, Error};

#[derive(Debug, Serialize)]
pub struct Table {
    name: String,
    columns: Vec<Column>,
}

#[derive(Debug, Serialize)]
pub struct Column {
    name: String,
    data_type: String,
}

pub async fn tables(schema: &str, client: &Client) -> Result<Vec<Table>, Error> {
    const QUERY: &str = "
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = $1 AND table_type = 'BASE TABLE'
    ";

    let rows = client.query(QUERY, &[&schema]).await?;
    let mut result = vec![];
    for row in rows.iter() {
        let table_name: &str = row.get("table_name");
        let columns = columns(table_name, schema, client).await?;
        result.push(Table {
            name: table_name.to_owned(),
            columns,
        });
    }
    Ok(result)
}

pub async fn columns(table: &str, schema: &str, client: &Client) -> Result<Vec<Column>, Error> {
    const QUERY: &str = "
        SELECT
            pg_attribute.attname AS column_name,
            pg_attribute.atttypid AS oid
        FROM
            pg_catalog.pg_attribute
        INNER JOIN
            pg_catalog.pg_class ON pg_class.oid = pg_attribute.attrelid
        INNER JOIN
            pg_catalog.pg_namespace ON pg_namespace.oid = pg_class.relnamespace
        WHERE
            pg_attribute.attnum > 0
            AND NOT pg_attribute.attisdropped
            AND pg_namespace.nspname = $2
            AND pg_class.relname = $1
    ";

    let rows = client.query(QUERY, &[&table, &schema]).await?;
    Ok(rows
        .iter()
        .map(|row| {
            let column_name: &str = row.get("column_name");
            let oid: Oid = row.get("oid");
            Column {
                name: column_name.to_owned(),
                data_type: Type::from_oid(oid).unwrap().name().to_owned(),
            }
        })
        .collect())
}
