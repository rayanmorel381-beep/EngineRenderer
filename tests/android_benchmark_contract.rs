use std::process::Command;

#[test]
#[ignore]
fn android_release_benchmark_reports_summary_line() {
    let out = Command::new("adb")
        .arg("shell")
        .arg("cd /data/local/tmp && ./enginerenderer_release run --fps 120 --width 1280 --height 720 --seconds 3")
        .output()
        .expect("failed to run adb shell command");

    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("realtime:"));
    assert!(stdout.contains("target_fps=120"));
    assert!(stdout.contains("achieved_fps="));
}
