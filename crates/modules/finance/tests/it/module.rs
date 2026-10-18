use std::sync::Arc;

use vitarchy_finance::{files::FileStore, reader::testing::FakeReader};

#[test]
fn the_module_is_finance_and_its_router_accepts_injected_dependencies() {
    assert_eq!(vitarchy_finance::module().id(), "finance");
    let _ = vitarchy_finance::router_with(
        Arc::new(FakeReader::default()),
        FileStore::new("/nonexistent"),
    );
}
