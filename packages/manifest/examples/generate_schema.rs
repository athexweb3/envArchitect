use env_manifest::EnhancedManifest;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(EnhancedManifest);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
