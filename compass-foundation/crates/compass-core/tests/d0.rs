use compass_core::{
    codec,
    d0::{self, Limits, Request},
};
use serde_json::Value;
use std::{fs, path::Path, sync::atomic::AtomicBool};
fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let p = entry.unwrap().path();
        let target = to.join(p.file_name().unwrap());
        if p.is_dir() {
            copy(&p, &target)
        } else {
            fs::copy(p, target).unwrap();
        }
    }
}
#[test]
fn frozen_physical_reports_exact_except_real_producer() {
    let fix = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../research/proposals/d0-c2/fixtures");
    let temp = tempfile::tempdir().unwrap();
    copy(&fix, temp.path());
    let fix = temp.path();
    let cases: Value =
        serde_json::from_slice(&fs::read(fix.join("report-inputs.json")).unwrap()).unwrap();
    for (name, c) in cases.as_object().unwrap() {
        let source = |side: &str| c[format!("{side}_package")].as_str().map(|s| fix.join(s));
        let req = Request {
            baseline: fix.join(c["baseline_manifest"].as_str().unwrap()),
            current: fix.join(c["current_manifest"].as_str().unwrap()),
            baseline_source: source("baseline"),
            current_source: source("current"),
            limits: Limits::default(),
        };
        let actual = d0::measure(&req, &AtomicBool::new(false)).unwrap();
        let mut expected: Value = serde_json::from_slice(
            &fs::read(fix.join(c["expected_report"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        expected["producer_build"] = Value::String(compass_core::BUILD_ID.into());
        expected["producer_version"] = Value::String(env!("CARGO_PKG_VERSION").into());
        assert_eq!(actual.value, expected, "{name}");
        assert_eq!(
            actual.exit_code,
            c["expected_exit"].as_i64().unwrap() as i32
        );
        assert_eq!(actual.bytes, codec::canonical(&actual.value).unwrap());
    }
}
