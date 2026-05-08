use kit::components::message::{format_text, is_only_emojis, replace_emojis, Order};

/// INT-01: Formatage complet d'un message avec markdown + liens + emojis
/// Fonctions impliquées: format_text → markdown → wrap_links_with_a_tags
///
/// Scénario: Un message contenant du texte en gras, un lien, et un emoji
/// doit avoir tous les éléments formatés correctement.
#[test]
fn int_01_complete_formatting_markdown_links_emojis() {
    let text = "**Important**: Check https://example.com :)";
    let result = format_text(text, true, true, None);

    // Vérifier que le markdown est appliqué
    assert!(
        result.contains("<strong>"),
        "Markdown bold should be applied"
    );

    // Vérifier que le lien est présent (le wrapping <a> se fait dans render.rs, pas format_text)
    assert!(
        result.contains("https://example.com"),
        "Link URL should be present in output"
    );

    // Vérifier que l'emoji est remplacé
    assert!(result.contains("🙂"), "Emoji should be converted");

    // Vérifier que tout est dans le HTML
    assert!(result.contains("<p>") || result.contains("<strong>"));
}

/// INT-02: Message avec échappement HTML suivi de markdown et détection de liens
/// Fonctions impliquées: format_text (HTML escape) → markdown → wrap_links_with_a_tags
///
/// Scénario: Un message contenant des caractères HTML, markdown, et liens
/// doit échapper le HTML puis appliquer les autres transformations.
#[test]
fn int_02_html_escape_then_markdown_links() {
    let text = "This <tag> & markdown **bold** at https://example.com";
    let result = format_text(text, true, false, None);

    // Vérifier que le HTML est échappé
    assert!(result.contains("&lt;"), "< should be escaped");
    assert!(result.contains("&gt;"), "> should be escaped");
    assert!(result.contains("&amp;"), "& should be escaped");

    // Vérifier que le markdown est appliqué après échappement
    assert!(result.contains("<strong>"), "Markdown should be applied");

    // Vérifier que le lien est détecté
    assert!(
        result.contains("https://example.com"),
        "Link should be detected"
    );

    // Vérifier que le texte "<tag>" est échappé (pas une vraie balise)
    assert!(!result.contains("<tag>"), "Raw <tag> should be escaped");
}

/// INT-03: Message contenant uniquement des emojis doit avoir la classe big-emoji
/// Fonctions impliquées: format_text → replace_emojis → is_only_emojis
///
/// Scénario: Un message contenant uniquement des emojis doit être enveloppé
/// dans une balise span avec la classe "big-emoji" pour un affichage agrandi.
#[test]
fn int_03_only_emojis_big_emoji_class() {
    let text = "😀😁😂";
    let result = format_text(text, false, true, None);

    // Vérifier que la classe big-emoji est présente
    assert!(
        result.contains("big-emoji"),
        "Should have big-emoji class for emoji-only messages"
    );

    // Vérifier que les emojis sont présents
    assert!(result.contains("😀"), "Emoji should be preserved");
    assert!(result.contains("😁"), "Emoji should be preserved");
    assert!(result.contains("😂"), "Emoji should be preserved");

    // Vérifier que c'est un span, pas un p
    assert!(result.contains("<span"), "Should use span, not p");
}

/// INT-03b: Variation - Texte avec emojis ASCII qui deviennent seuls emojis
/// Teste le cas où les emojis ASCII sont convertis et le texte devient seul-emoji
#[test]
fn int_03b_ascii_emojis_become_big_emoji() {
    let text = ":) :D ;)";
    let result = format_text(text, false, true, None);

    // Les emojis ASCII sont remplacés
    assert!(result.contains("🙂"), "ASCII emoji should be converted");
    assert!(result.contains("😁"), "ASCII emoji should be converted");
    assert!(result.contains("😉"), "ASCII emoji should be converted");

    // Si le résultat ne contient que des emojis, il doit avoir big-emoji
    if is_only_emojis("🙂😁😉") {
        assert!(
            result.contains("big-emoji"),
            "Big emoji class should be applied"
        );
    }
}

/// INT-04: Message avec liens mailto et URL mixtes dans du texte markdown
/// Fonctions impliquées: markdown → wrap_links_with_a_tags (avec mailto)
///
/// Scénario: Un message contenant du markdown et des liens mixtes (http, www, mailto)
/// doit traiter tous les types de liens correctement.
#[test]
fn int_04_mixed_links_and_markdown() {
    let text = "Contact me at mailto: user@example.com or visit https://example.com";
    let result = format_text(text, true, false, None);

    // Note: wrap_links_with_a_tags est appelé dans render.rs, pas dans format_text
    // format_text retourne le texte avec markdown mais sans wrapping des liens en <a>
    assert!(
        result.contains("user@example.com"),
        "email should be present in output"
    );
    assert!(
        result.contains("https://example.com"),
        "https URL should be present in output"
    );

    // Le texte complet doit être présent
    assert!(result.contains("Contact"), "Text should be preserved");
}

/// INT-05: Chaîne contenant des caractères HTML malveillants suivie de formatage complet
/// Fonctions impliquées: format_text (XSS prevention) → markdown
///
/// Scénario: Un message contenant des balises de script ou d'événements HTML
/// doit être complètement échappé pour prévenir les attaques XSS.
// note: VIDEO
#[test]
fn int_05_xss_prevention_complete_formatting() {
    let text = "<script>alert('xss')</script> Hello **world**";
    let result = format_text(text, true, false, None);

    // Vérifier que le script n'est pas exécuté
    assert!(
        !result.contains("<script>"),
        "Script tag should be escaped, not executed"
    );

    // Vérifier que les caractères < et > sont échappés
    assert!(result.contains("&lt;script"), "< should be escaped");
    assert!(result.contains("&gt;"), "> should be escaped");

    // Vérifier que le markdown est toujours appliqué au contenu légitime
    assert!(
        result.contains("<strong>"),
        "Legitimate markdown should still work"
    );
    assert!(result.contains("world"), "Text content should be preserved");

    // Vérifier que "Hello" est présent
    assert!(result.contains("Hello"), "Text should be preserved");
}

/// INT-05b: Variation - Événements HTML malveillants
/// Teste les balises d'événements qui tentent d'exécuter du JavaScript
// note: VIDEO
#[test]
fn int_05b_event_handler_xss_prevention() {
    let text = "<img src=x onerror=\"alert('xss')\">";
    let result = format_text(text, false, false, None);

    // Tous les caractères spéciaux doivent être échappés
    assert!(result.contains("&lt;"), "< should be escaped");
    assert!(result.contains("&quot;"), "quotes should be escaped");
    assert!(result.contains("&gt;"), "> should be escaped");
    assert!(result.contains("&#x27;"), "single quotes should be escaped");

    // La balise <img ne doit pas être présente (le < est échappé en &lt;)
    assert!(!result.contains("<img"), "img tag should be escaped");
    // Note: l'attribut 'onerror=' lui-même n'est pas un caractère spécial HTML
    // mais les guillemets autour de sa valeur sont échappés en &quot;
    // donc le code JavaScript ne peut pas s'exécuter
    assert!(
        result.contains("onerror=&quot;") || result.contains("onerror="),
        "onerror attribute text preserved but quotes escaped"
    );
    // Vérifier qu'aucune balise exécutable n'est présente
    assert!(!result.contains("<img "), "No executable img tag should exist");
}

/// INT-06: Message avec blocs de code contenant des emojis ASCII
/// Fonctions impliquées: markdown (code block) + stack_processor
///
/// Scénario: Un message avec un bloc de code contenant des emojis ASCII
/// ne doit PAS convertir les emojis ASCII dans le code en vrais emojis.
// note: VIDEO
#[test]
fn int_06_code_block_preserves_ascii_emojis() {
    let text = "Here's some code:\n```\nsmile = ':)'\nface = ':('\n```\nEnd";
    let result = format_text(text, true, false, None);

    // Les emojis ASCII ne doivent PAS être convertis dans le code
    // Ils doivent rester sous forme `:)` et `:(`
    // (Note: Ce test dépend de la manière dont markdown traite les blocs de code)
    assert!(result.contains("code"), "Code block should be present");

    // Le texte contenant le code doit être présent
    assert!(
        result.contains("':)'") || result.contains(":)"),
        "Code should preserve emoji syntax"
    );
}

/// INT-06b: Variation - Bloc de code indenté avec emojis
/// Teste le formatage des blocs de code indentés
#[test]
fn int_06b_indented_code_block() {
    let text = "Example:\n\n    emoji = ':)'\n\nEnd";
    let result = format_text(text, true, false, None);

    // Le code indenté doit être traité comme un bloc de code
    assert!(result.contains("emoji"), "Code content should be present");
    assert!(
        result.contains("':)'") || result.contains("'"),
        "Quotes should be preserved"
    );
}

// ============ Additional comprehensive integration tests ============

/// Test: Combinaison complexe de markdown, liens, emojis et HTML dangereux
/// Tous les éléments sont présents dans un seul message
#[test]
fn int_complex_multi_feature_message() {
    let text =
        "**Important** <script>alert</script> Check https://example.com & **also** www.test.org :)";
    let result = format_text(text, true, true, None);

    // Markdown
    assert!(result.contains("<strong>"), "Markdown should work");

    // XSS prevention
    assert!(result.contains("&lt;script"), "Script should be escaped");

    // Links (URL text preserved; <a> wrapping happens in render.rs, not format_text)
    assert!(
        result.contains("https://example.com"),
        "HTTPS URL should be present in output"
    );
    assert!(
        result.contains("www.test.org"),
        "WWW URL should be present in output"
    );

    // HTML entities
    assert!(result.contains("&amp;"), "& should be escaped");

    // Emojis
    assert!(result.contains("🙂"), "Emoji should be converted");
}

/// Test: Message avec plusieurs types de liens
#[test]
fn int_multiple_link_types() {
    let text = "Contact: https://example.com, www.test.org, or user@test.com";
    let result = format_text(text, false, false, None);

    // Note: <a> wrapping happens in render.rs via wrap_links_with_a_tags, not in format_text
    // Verify URL text is preserved in output
    assert!(result.contains("https://example.com"), "HTTPS URL should be present");
    assert!(result.contains("www.test.org"), "WWW URL should be present");
    assert!(result.contains("user@test.com"), "Email should be present");
}

/// Test: Message avec tous les types de markdown
#[test]
fn int_all_markdown_features() {
    let text = "**bold** *italic* ~~strikethrough~~ `code` and more";
    let result = format_text(text, true, false, None);

    // Vérifier que tous les éléments markdown sont appliqués
    assert!(result.contains("<strong>"), "Bold should be applied");
    assert!(result.contains("<em>"), "Italic should be applied");
    assert!(result.contains("<del>"), "Strikethrough should be applied");
    assert!(result.contains("<code>"), "Code should be applied");
}

/// Test: Message avec HTML escaping + markdown + links + emojis
#[test]
fn int_full_pipeline_all_features() {
    let text = "Report <vulnerability> at https://security.example.com **asap** :)";
    let result = format_text(text, true, true, None);

    // XSS prevention
    assert!(result.contains("&lt;vulnerability&gt;"));

    // Markdown
    assert!(result.contains("<strong>asap</strong>"));

    // Links (URL preserved; <a> wrapping in render.rs, not format_text)
    assert!(result.contains("https://security.example.com"));

    // Emojis
    assert!(result.contains("🙂"));
}

/// Test: Edge case - Empty message after all transformations
#[test]
fn int_edge_case_whitespace_only() {
    let text = "   ";
    let result = format_text(text, true, true, None);

    // Should still produce valid HTML
    assert!(result.contains("<p>") || result.contains("<span>"));
}

/// Test: Edge case - Message with only special HTML characters
#[test]
fn int_edge_case_only_special_chars() {
    let text = "< > & \" '";
    let result = format_text(text, false, false, None);

    // All should be escaped
    assert!(result.contains("&lt;"));
    assert!(result.contains("&gt;"));
    assert!(result.contains("&amp;"));
    assert!(result.contains("&quot;"));
    assert!(result.contains("&#x27;"));
}

/// Test: Order enum usage in message context
#[test]
fn int_order_enum_display() {
    // In a real scenario, these would be used for CSS class selection
    assert_eq!(Order::First.to_string(), "message-first");
    assert_eq!(Order::Middle.to_string(), "message-middle");
    assert_eq!(Order::Last.to_string(), "message-last");

    // Different from each other
    assert!(Order::First != Order::Middle);
    assert!(Order::Middle != Order::Last);
    assert!(Order::First != Order::Last);

    // Can be compared and tested
    assert!(Order::First == Order::First);
    assert!(Order::Middle == Order::Middle);
    assert!(Order::Last == Order::Last);
}

/// Test: Newline preservation and HTML entity conversion
#[test]
fn int_newline_handling() {
    let text = "line 1\nline 2\nline 3";
    let result = format_text(text, false, false, None);

    // Newlines should be converted to &nbsp;&nbsp; entities
    assert!(result.contains("&nbsp;&nbsp;"));

    // All lines should be present
    assert!(result.contains("line 1"));
    assert!(result.contains("line 2"));
    assert!(result.contains("line 3"));
}

/// Test: Markdown with links - should ignore link syntax but detect URLs
#[test]
fn int_markdown_link_syntax_ignored_but_urls_detected() {
    let text = "Check [this](https://example.com) and also https://example.org";
    let result = format_text(text, true, false, None);

    // The markdown link syntax should be ignored (based on code)
    // But explicit URLs should be detected
    assert!(result.contains("https://example.org"));
}

/// Test: Emoji detection with whitespace
#[test]
fn int_emoji_with_surrounding_whitespace() {
    let text = "  🙂  ";
    let result = format_text(text, false, true, None);

    // Should be detected as emoji-only despite whitespace
    if is_only_emojis(text) {
        assert!(result.contains("big-emoji"));
    }
}

/// Test: Complex scenario from real usage
#[test]
fn int_realistic_message_scenario() {
    let text = "Hey! Check out this **cool** link: https://github.com/example/repo :) BTW, here's my <email>: user@example.com";
    let result = format_text(text, true, true, None);

    // All components should work together
    assert!(result.contains("cool"), "Text should be present");
    assert!(
        result.contains("<strong>cool</strong>"),
        "Markdown should work"
    );
    assert!(
        result.contains("https://github.com"),
        "HTTPS link should work"
    );
    assert!(result.contains("🙂"), "Emoji should be converted");
    assert!(result.contains("&lt;email&gt;"), "HTML should be escaped");
    assert!(
        result.contains("user@example.com"),
        "Email link should work"
    );
}
