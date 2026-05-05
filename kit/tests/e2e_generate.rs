use kit::components::message::format_text;
use std::fs;
use std::path::PathBuf;

/// E2E HTML generator — runs the FULL message rendering pipeline in Rust,
/// writes the result to an HTML file that Playwright will load and verify.
///
/// Pipeline tested: raw text → HTML escape → markdown → emoji replacement
///
/// Run with: cargo test -p kit --test e2e_generate -- --nocapture
#[test]
fn generate_e2e_html() {
    let test_cases = vec![
        (
            "realistic-message",
            "**Important**: Visit https://example.com & say hello :)",
            true,  // markdown
            true,  // emojis
        ),
        (
            "xss-prevention",
            "<script>alert('xss')</script> **safe** content",
            true,
            false,
        ),
        (
            "emoji-only",
            ":) :D ;)",
            false,
            true,
        ),
        (
            "plain-text",
            "Just a simple message with no formatting",
            false,
            false,
        ),
    ];

    let mut html_sections = String::new();
    for (id, raw, markdown, emojis) in &test_cases {
        // Full pipeline: HTML escape + optional markdown + optional emoji
        let final_html = format_text(raw, *markdown, *emojis, None);

        html_sections.push_str(&format!(
            "  <div id=\"{}\" class=\"message\" data-raw=\"{}\">{}</div>\n",
            id,
            raw.replace('"', "&quot;"),
            final_html,
        ));
    }

    let page = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>E2E Message Rendering</title></head>
<body>
{}
</body>
</html>"#,
        html_sections
    );

    // Write to automated-e2e directory (parent of Uplink)
    // CARGO_MANIFEST_DIR = .../Uplink/kit → parent = .../Uplink → parent = .../labo-2
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("automated-e2e")
        .join("test-output.html");

    fs::write(&output_path, &page).unwrap_or_else(|e| {
        panic!("Failed to write {}: {}", output_path.display(), e);
    });

    println!("Generated: {}", output_path.display());
}
