use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    println!("====================================");
    println!("    RUST DATA ENGINEERING PIPELINE");
    println!("====================================");

    let data_folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("raw");

    println!("\nStarting data ingestion...\n");

    inspect_csv(&data_folder.join("customers.csv"))?;
    inspect_csv(&data_folder.join("products.csv"))?;
    inspect_csv(&data_folder.join("orders.csv"))?;

    println!("\n====================================");
    println!("Data ingestion completed!");
    println!("====================================");
    Ok(())


}

fn inspect_csv(file_path: &Path) -> Result<(), Box<dyn Error>> {
    let file_name = file_path
        .file_name()
        .unwrap()
        .to_string_lossy();
    println!("Processing file: {}", file_name);
    let mut reader = csv::Reader::from_path(file_path)?;

    // Read CSV column names
    let headers = reader.headers()?.clone();

    println!("Columns: {}", headers.len());
    println!("Headers: {:?}", headers);

    // Count data records
    let mut row_count = 0;

    for record in reader.records() {
        let _record = record?;
        row_count += 1;
    }

    println!("Records: {}", row_count);
    println!("------------------------------------");

    Ok(())
}