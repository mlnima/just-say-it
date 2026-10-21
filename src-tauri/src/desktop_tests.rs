#![cfg(windows)]

use crate::platform::Focus;
use dictation_core::session::Delivery;
use std::os::windows::process::CommandExt;
use std::{
    path::Path,
    process::{Child, Command},
    sync::atomic::AtomicU64,
    time::{Duration, Instant},
};

struct Editor(Child);
impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
        crate::fixture_focus::restore();
    }
}

fn wait_until(label: &str, mut check: impl FnMut() -> bool) {
    let deadline =
        Instant::now() + Duration::from_secs(if label == "fixture ready" { 30 } else { 5 });
    while !check() {
        assert!(
            Instant::now() < deadline,
            "Desktop fixture timed out: {label}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn action(directory: &Path, value: serde_json::Value) {
    let temporary = directory.join("command.part");
    std::fs::write(&temporary, serde_json::to_vec(&value).unwrap()).unwrap();
    std::fs::rename(temporary, directory.join("command.json")).unwrap();
    wait_until("command consumed", || {
        !directory.join("command.json").exists()
    });
}

#[test]
#[ignore = "opens a dedicated real Windows editor fixture; requires an unlocked desktop"]
fn unicode_delivery_target_changes_and_protected_fields() {
    let directory = tempfile::tempdir().unwrap();
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/desktop/input-fixture.ps1");
    let _editor = Editor(
        Command::new("powershell.exe")
            .args(["-NoProfile", "-STA", "-File"])
            .arg(script)
            .arg("-Directory")
            .arg(directory.path())
            .creation_flags(0x08000000)
            .spawn()
            .unwrap(),
    );
    wait_until("fixture ready", || directory.path().join("ready").exists());
    crate::fixture_focus::first_editor(_editor.0.id());
    let focus = Focus::new().unwrap();
    wait_until("editable fixture input focused", || {
        let target = focus.inspect();
        target.known() && target.process_id == _editor.0.id()
    });
    let target = focus.inspect();
    let authorization = AtomicU64::new(1);
    let expected = "Grüße 👋 مرحبا";
    assert_eq!(
        focus.insert(&target, expected, &authorization, 1),
        Delivery::Submitted
    );
    action(
        directory.path(),
        serde_json::json!({ "action": "focus", "index": 1 }),
    );
    wait_until("second field focused", || !target.same(&focus.inspect()));
    assert_eq!(
        focus.insert(&target, "must not appear", &authorization, 1),
        Delivery::Rejected
    );
    action(directory.path(), serde_json::json!({ "action": "read" }));
    wait_until("result received", || {
        directory.path().join("result.json").exists()
    });
    let json = std::fs::read_to_string(directory.path().join("result.json")).unwrap();
    let result: serde_json::Value =
        serde_json::from_str(json.trim_start_matches('\u{feff}')).unwrap();
    assert_eq!(result["first"], expected);
    assert_eq!(result["second"], "");
    action(
        directory.path(),
        serde_json::json!({ "action": "focus", "index": 2 }),
    );
    wait_until("protected field focused", || focus.inspect().secure);
    let password = focus.inspect();
    assert!(!password.known());
    assert_eq!(
        focus.insert(&password, "refused", &authorization, 1),
        Delivery::Rejected
    );
    action(
        directory.path(),
        serde_json::json!({ "action": "focus", "index": 3 }),
    );
    wait_until("read only field focused", || {
        !focus.inspect().secure && !focus.inspect().editable
    });
    action(directory.path(), serde_json::json!({ "action": "close" }));
}

#[test]
#[ignore = "requires Microsoft Edge, Playwright and an unlocked desktop"]
fn browser_inputs_and_contenteditable_receive_unicode() {
    let directory = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let browser = Editor(
        Command::new("node")
            .arg("tests/desktop/browser-fixture.mjs")
            .arg(directory.path())
            .current_dir(root)
            .creation_flags(0x08000000)
            .spawn()
            .unwrap(),
    );
    wait_until("fixture ready", || directory.path().join("ready").exists());
    let processes: Vec<u32> =
        serde_json::from_str(&std::fs::read_to_string(directory.path().join("ready")).unwrap())
            .unwrap();
    let focus = Focus::new().unwrap();
    wait_until("browser fixture focused", || {
        let target = focus.inspect();
        target.known() && processes.contains(&target.process_id)
    });
    let target = focus.inspect();
    let authorization = AtomicU64::new(1);
    let expected = "Grüße 👋 مرحبا";
    assert!(
        target.same(&focus.inspect()),
        "Fixture target changed before insertion."
    );
    crate::fixture_focus::park(
        windows::Win32::Foundation::POINT {
            x: (target.bounds.left + target.bounds.right) / 2,
            y: (target.bounds.top + target.bounds.bottom) / 2,
        },
        &processes,
    );
    assert_eq!(
        focus.insert(&target, expected, &authorization, 1),
        Delivery::Submitted
    );
    action(
        directory.path(),
        serde_json::json!({ "action": "focus", "id": "second" }),
    );
    wait_until("browser field changed", || !target.same(&focus.inspect()));
    assert_eq!(
        focus.insert(&target, "must not appear", &authorization, 1),
        Delivery::Rejected
    );
    action(
        directory.path(),
        serde_json::json!({ "action": "focus", "id": "editable" }),
    );
    wait_until("contenteditable focused", || {
        let target = focus.inspect();
        target.known() && processes.contains(&target.process_id)
    });
    assert_eq!(
        focus.insert(&focus.inspect(), expected, &authorization, 1),
        Delivery::Submitted
    );
    action(directory.path(), serde_json::json!({ "action": "read" }));
    let result: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(directory.path().join("result.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(result["first"], expected);
    assert_eq!(result["second"], "");
    assert_eq!(result["editable"], expected);
    action(directory.path(), serde_json::json!({ "action": "close" }));
    drop(browser);
}

#[test]
#[ignore = "opens real WPF document controls; requires an unlocked desktop"]
fn document_inputs_receive_unicode_and_refuse_read_only_or_changed_targets() {
    let directory = tempfile::tempdir().unwrap();
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/desktop/document-fixture.ps1");
    let editor = Editor(
        Command::new("powershell.exe")
            .args(["-NoProfile", "-STA", "-File"])
            .arg(script)
            .arg("-Directory")
            .arg(directory.path())
            .creation_flags(0x08000000)
            .spawn()
            .unwrap(),
    );
    wait_until("fixture ready", || directory.path().join("ready").exists());
    let focus = Focus::new().unwrap();
    wait_until("document input focused", || {
        let target = focus.inspect();
        target.known() && target.process_id == editor.0.id()
    });
    let target = focus.inspect();
    let authorization = AtomicU64::new(1);
    let expected = "Grüße 👋 مرحبا";
    assert_eq!(
        focus.insert(&target, expected, &authorization, 1),
        Delivery::Submitted
    );
    action(
        directory.path(),
        serde_json::json!({ "action": "focus", "index": 1 }),
    );
    wait_until("document field changed", || !target.same(&focus.inspect()));
    assert_eq!(
        focus.insert(&target, "refused", &authorization, 1),
        Delivery::Rejected
    );
    action(directory.path(), serde_json::json!({ "action": "read" }));
    let json = std::fs::read_to_string(directory.path().join("result.json")).unwrap();
    let result: serde_json::Value =
        serde_json::from_str(json.trim_start_matches('\u{feff}')).unwrap();
    assert_eq!(result["first"], expected);
    assert_eq!(result["second"], "");
    action(
        directory.path(),
        serde_json::json!({ "action": "focus", "index": 2 }),
    );
    wait_until("read only document focused", || {
        let current = focus.inspect();
        current.process_id == editor.0.id() && !current.secure && !current.editable
    });
    assert!(!focus.inspect().known());
    assert_eq!(
        focus.insert(&focus.inspect(), "refused", &authorization, 1),
        Delivery::Rejected
    );
    action(directory.path(), serde_json::json!({ "action": "close" }));
}
