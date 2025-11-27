//! Visual snapshot test scaffold for gpui-hn-app
//!
//! This test is intentionally ignored by default. It documents how to run a
//! headless renderer to capture a screenshot of the layout for manual or
//! automated pixel-comparison against the freya-hn-app reference.
//!
//! To enable these tests you need an environment capable of running a headless
//! GPU or software renderer and a small harness to capture the window surface.
//! Implementation is intentionally left as a scaffold - enable it locally when
//! you have the required headless environment.

#[test]
#[ignore]
fn visual_snapshot_scaffold() {
    // Scaffold: steps to run locally
    // 1) Build the app in a headless-capable environment.
    // 2) Run the binary with an env var (e.g., GPUI_HEADLESS=1) that instructs the
    //    app to render a single frame and write a PNG to disk (e.g., target/snapshots/last.png).
    // 3) Use an image diff tool (compare, ImageMagick `compare`, or a CI snapshot tool)
    //    to compare the generated PNG with the `tests/reference/*.png` images.

    // The test is a placeholder and intentionally succeeds when executed (ignored).
    // Run the scaffold manually when headless renderer is available
}
