
use csv;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

// ==========================================
// 1. DATA STRUCTURES
// ==========================================

#[derive(Debug, Deserialize, Serialize)]
struct Customer {
    customer_id: String,
    first_name: String,
    last_name: String,
    email: String,
    country: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Product {
    product_id: String,
    product_name: String,
    category: String,
    price: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Order {
    order_id: String,
    customer_id: String,
    product_id: String,
    quantity: String,
    order_date: String,
}

// ==========================================
// 2. DATA QUALITY COUNTERS
// ==========================================

struct QualityReport {
    total: usize,
    accepted: usize,
    rejected: usize,
    reasons: Vec<String>,
}

impl QualityReport {
    fn new() -> Self {
        Self {
            total: 0,
            accepted: 0,
            rejected: 0,
            reasons: Vec::new(),
        }
    }

    fn reject(&mut self, reason: String) {
        self.rejected += 1;
        self.reasons.push(reason);
    }
}

// ==========================================
// 3. GENERIC CSV READER
// ==========================================

fn read_csv<T: DeserializeOwned>(
    path: &Path,
) -> Result<Vec<T>, Box<dyn Error>> {

    let mut reader = csv::Reader::from_path(path)?;

    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: T = result?;
        records.push(record);
    }

    Ok(records)
}

// ==========================================
// 4. CSV WRITER
// ==========================================

fn write_csv<T: Serialize>(
    path: &Path,
    records: &[T],
) -> Result<(), Box<dyn Error>> {

    let mut writer = csv::Writer::from_path(path)?;

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;

    Ok(())
}

// ==========================================
// 5. CUSTOMER VALIDATION
// ==========================================

fn validate_customers(
    customers: Vec<Customer>,
) -> (Vec<Customer>, QualityReport, HashSet<String>) {

    let mut valid_customers = Vec::new();
    let mut customer_ids = HashSet::new();
    let mut report = QualityReport::new();

    for customer in customers {

        report.total += 1;

        if customer.customer_id.trim().is_empty()
            || customer.first_name.trim().is_empty()
            || customer.last_name.trim().is_empty()
            || customer.country.trim().is_empty()
        {
            report.reject(format!(
                "Customer {}: missing required field",
                customer.customer_id
            ));
            continue;
        }

        if !customer.email.contains('@')
            || !customer.email.split('@').nth(1)
            .map(|domain| domain.contains('.'))
            .unwrap_or(false)
        {
            report.reject(format!(
                "Customer {}: invalid email",
                customer.customer_id
            ));
            continue;
        }

        if !customer_ids.insert(customer.customer_id.clone()) {
            report.reject(format!(
                "Customer {}: duplicate ID",
                customer.customer_id
            ));
            continue;
        }

        valid_customers.push(customer);
        report.accepted += 1;
    }

    (valid_customers, report, customer_ids)
}

// ==========================================
// 6. PRODUCT VALIDATION
// ==========================================

fn validate_products(
    products: Vec<Product>,
) -> (Vec<Product>, QualityReport, HashSet<String>) {

    let mut valid_products = Vec::new();
    let mut product_ids = HashSet::new();
    let mut report = QualityReport::new();

    for product in products {

        report.total += 1;

        if product.product_id.trim().is_empty()
            || product.product_name.trim().is_empty()
            || product.category.trim().is_empty()
        {
            report.reject(format!(
                "Product {}: missing required field",
                product.product_id
            ));
            continue;
        }

        let price: f64 = match product.price.parse() {
            Ok(value) => value,
            Err(_) => {
                report.reject(format!(
                    "Product {}: invalid price",
                    product.product_id
                ));
                continue;
            }
        };

        if !price.is_finite() || price <= 0.0 {
            report.reject(format!(
                "Product {}: price must be greater than zero",
                product.product_id
            ));
            continue;
        }

        if !product_ids.insert(product.product_id.clone()) {
            report.reject(format!(
                "Product {}: duplicate ID",
                product.product_id
            ));
            continue;
        }

        valid_products.push(product);
        report.accepted += 1;
    }

    (valid_products, report, product_ids)
}

// ==========================================
// 7. ORDER VALIDATION
// ==========================================

fn validate_orders(
    orders: Vec<Order>,
    customer_ids: &HashSet<String>,
    product_ids: &HashSet<String>,
) -> (Vec<Order>, QualityReport) {

    let mut valid_orders = Vec::new();
    let mut order_ids = HashSet::new();
    let mut report = QualityReport::new();

    for order in orders {

        report.total += 1;

        if order.order_id.trim().is_empty()
            || order.customer_id.trim().is_empty()
            || order.product_id.trim().is_empty()
            || order.order_date.trim().is_empty()
        {
            report.reject(format!(
                "Order {}: missing required field",
                order.order_id
            ));
            continue;
        }

        let quantity: u32 = match order.quantity.parse() {
            Ok(value) => value,
            Err(_) => {
                report.reject(format!(
                    "Order {}: invalid quantity",
                    order.order_id
                ));
                continue;
            }
        };

        if quantity == 0 {
            report.reject(format!(
                "Order {}: quantity must be greater than zero",
                order.order_id
            ));
            continue;
        }

        if !customer_ids.contains(&order.customer_id) {
            report.reject(format!(
                "Order {}: customer does not exist",
                order.order_id
            ));
            continue;
        }

        if !product_ids.contains(&order.product_id) {
            report.reject(format!(
                "Order {}: product does not exist",
                order.order_id
            ));
            continue;
        }

        if !order_ids.insert(order.order_id.clone()) {
            report.reject(format!(
                "Order {}: duplicate ID",
                order.order_id
            ));
            continue;
        }

        valid_orders.push(order);
        report.accepted += 1;
    }

    (valid_orders, report)
}

// ==========================================
// 8. DATA QUALITY REPORT
// ==========================================

fn print_report(name: &str, report: &QualityReport) {

    println!("\n========== {} ==========", name);

    println!("Total records: {}", report.total);
    println!("Accepted records: {}", report.accepted);
    println!("Rejected records: {}", report.rejected);

    if !report.reasons.is_empty() {
        println!("\nValidation issues:");

        for reason in &report.reasons {
            println!(" - {}", reason);
        }
    }
}

// ==========================================
// 9. MAIN PIPELINE
// ==========================================

fn main() -> Result<(), Box<dyn Error>> {

    println!("====================================");
    println!("     RUST DATA QUALITY PIPELINE");
    println!("====================================");

    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let raw_dir = project_dir.join("data").join("raw");
    let processed_dir = project_dir.join("data").join("processed");

    fs::create_dir_all(&processed_dir)?;

    // STEP 1: READ DATA

    println!("\nSTEP 1: Reading CSV files...");

    let customers: Vec<Customer> =
        read_csv(&raw_dir.join("customers.csv"))?;

    let products: Vec<Product> =
        read_csv(&raw_dir.join("products.csv"))?;

    let orders: Vec<Order> =
        read_csv(&raw_dir.join("orders.csv"))?;

    // STEP 2: VALIDATE CUSTOMERS

    println!("\nSTEP 2: Validating customers...");

    let (valid_customers, customer_report, customer_ids) =
        validate_customers(customers);

    // STEP 3: VALIDATE PRODUCTS

    println!("\nSTEP 3: Validating products...");

    let (valid_products, product_report, product_ids) =
        validate_products(products);

    // STEP 4: VALIDATE ORDERS

    println!("\nSTEP 4: Validating orders...");

    let (valid_orders, order_report) =
        validate_orders(orders, &customer_ids, &product_ids);

    // STEP 5: SAVE PROCESSED DATA

    println!("\nSTEP 5: Saving processed data...");

    write_csv(
        &processed_dir.join("customers_clean.csv"),
        &valid_customers,
    )?;

    write_csv(
        &processed_dir.join("products_clean.csv"),
        &valid_products,
    )?;

    write_csv(
        &processed_dir.join("orders_clean.csv"),
        &valid_orders,
    )?;

    // STEP 6: GENERATE REPORT

    print_report("CUSTOMERS", &customer_report);
    print_report("PRODUCTS", &product_report);
    print_report("ORDERS", &order_report);

    let report_text = format!(
        "RUST DATA QUALITY REPORT\n\
         ========================\n\n\
         CUSTOMERS\n\
         Total: {}\n\
         Accepted: {}\n\
         Rejected: {}\n\n\
         PRODUCTS\n\
         Total: {}\n\
         Accepted: {}\n\
         Rejected: {}\n\n\
         ORDERS\n\
         Total: {}\n\
         Accepted: {}\n\
         Rejected: {}\n",
        customer_report.total,
        customer_report.accepted,
        customer_report.rejected,
        product_report.total,
        product_report.accepted,
        product_report.rejected,
        order_report.total,
        order_report.accepted,
        order_report.rejected,
    );

    fs::write(
        processed_dir.join("data_quality_report.txt"),
        report_text,
    )?;

    println!("\n====================================");
    println!("DATA QUALITY PIPELINE COMPLETED!");
    println!("====================================");

    println!(
        "Processed files saved in: {}",
        processed_dir.display()
    );

    Ok(())
}
