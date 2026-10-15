fn main() {
    if std::env::args().nth(1).as_deref() == Some("capabilities") {
        println!(
            r#"{{"protocol":3,"artifact":"vault","native_version":3,"migration_schema":3,"container_version":3,"structure_versions":[3],"migration_schemas":[3]}}"#
        );
    } else {
        eprintln!("obsolete exporter must never receive migration data");
        std::process::exit(91);
    }
}
