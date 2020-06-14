use args::Args;
use getopts::Occur;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use tokio_postgres::{Error, NoTls};

const NAME: &str = "Archon Toilet";
const DESCRIPTION: &str = "This tool generates rust code from a PostgreSQL database.";

async fn generate(
    database_string: &str,
    templates_folder: &Path,
    output_folder: &Path,
    schema: &str,
) -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect(database_string, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let mut file = File::open(templates_folder.join("tables.handlebars")).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    fs::write(
        output_folder.join("tables.rs"),
        archon_toilet::render(
            archon_toilet::tables(schema, &client).await.unwrap(),
            &contents,
        )
        .unwrap(),
    )
    .unwrap();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut args = Args::new(NAME, DESCRIPTION);
    args.option(
        "o",
        "output-folder",
        "The folder where generated files will be output.",
        "OUTPUT-FOLDER",
        Occur::Req,
        None,
    );
    args.option(
        "t",
        "templates-folder",
        "The folder containing the templates to be used for generation.",
        "TEMPLATES-FOLDER",
        Occur::Req,
        None,
    );
    args.option(
        "d",
        "database-string",
        "The database connection string, e.g. \"host=localhost user=postgres dbname=somedb\"",
        "DATABASE-STRING",
        Occur::Req,
        None,
    );
    args.option(
        "s",
        "schema",
        "The PostgreSQL schema to generate code from.",
        "SCHEMA",
        Occur::Optional,
        Some("".to_owned()),
    );

    args.parse(std::env::args()).unwrap();

    generate(
        &args.value_of::<String>("database-string").unwrap(),
        &args.value_of::<PathBuf>("templates-folder").unwrap(),
        &args.value_of::<PathBuf>("output-folder").unwrap(),
        &args.value_of::<String>("schema").unwrap(),
    )
    .await
}
