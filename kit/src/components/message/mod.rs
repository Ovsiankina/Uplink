use std::borrow::Cow;

use common::state::utils::{mention_replacement_pattern, parse_mentions};
use common::state::State;
use derive_more::Display;
use once_cell::sync::Lazy;
use pulldown_cmark::{CodeBlockKind, Options, Tag, TagEnd};
use regex::{Captures, Regex, Replacer};
use uuid::Uuid;

pub mod render;
pub use render::{
    ChatMessageProps, ChatText, IdentityCmd, IdentityMessage, IdentityMessageProps, Message, Props,
};

pub static MARKDOWN_PROCESSOR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(^|\n)((?:&gt;(?: *&gt;)*)|(?: ))").unwrap());
pub static LINK_TAGS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"((?:(?:www\.)|(?:https?:\/\/))[\w-]+(?:\.[\w-]+)+(?:\/[^)\s<]*)*)|((mailto: {0,1})([\w.+-]+@[\w-]+(?:\.[\w.-]+)+))").unwrap()
});

pub(crate) const HTML_ESCAPES: [(&str, &str); 5] = [
    ("&", "&amp;"),
    ("<", "&lt;"),
    (">", "&gt;"),
    ("\"", "&quot;"),
    ("\'", "&#x27;"),
];

#[derive(Eq, PartialEq, Clone, Copy, Display)]
pub enum Order {
    #[display(fmt = "message-first")]
    First,

    #[display(fmt = "message-middle")]
    Middle,

    #[display(fmt = "message-last")]
    Last,
}

#[derive(Eq, PartialEq, Clone)]
pub struct ReactionAdapter {
    pub emoji: String,
    pub alt: String,
    pub self_reacted: bool,
    pub reaction_count: usize,
}

// Struct for replacing links with clickable divs.
// Also saves the links
struct LinkReplacer(Vec<String>);

impl Replacer for LinkReplacer {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        let mut url = caps.get(0).unwrap().as_str().to_string();
        if url.starts_with("mailto:") {
            let s = if url.starts_with("mailto: ") {
                format!("{}<a href=\"{}\">{}</a>", &caps[3], url, &caps[4])
            } else {
                format!("<a href=\"{}\">{}</a>", url, url)
            };
            dst.push_str(&s);
            return;
        }
        // Check if it ends with a ) and exclude it if its not part of url
        while url.ends_with(')') {
            let count = url.chars().count();
            let open = url.chars().filter(|c| *c == '(').count();
            let close = url.chars().filter(|c| *c == ')').count();
            if close > open {
                url = url.chars().take(count - 1).collect::<String>();
            } else {
                break;
            }
        }
        let s = if url.starts_with("www.") {
            let html = format!("<a href=\"https://{}\">{}</a>", url, url);
            url = format!("https://{}", url);
            html
        } else {
            format!("<a href=\"{}\">{}</a>", url, url)
        };
        self.0.push(url);
        dst.push_str(&s);
    }
}

pub(crate) fn wrap_links_with_a_tags(text: &str) -> (String, Vec<String>) {
    let mut links = LinkReplacer(vec![]);
    let res = LINK_TAGS_REGEX
        .replace_all(text, links.by_ref())
        .into_owned();
    (res, links.0)
}

pub fn format_text(
    text: &str,
    should_markdown: bool,
    emojis: bool,
    data: Option<(&State, &Uuid, bool)>,
) -> String {
    // warning: this will probably break markdown regarding block quotes. still seems like an improvement.
    let safe_text = HTML_ESCAPES
        .iter()
        .fold(Cow::from(text), |s, (from, to)| s.replace(*from, to).into())
        .replace('\n', "&nbsp;&nbsp;\n");
    let mut text = safe_text;
    // We want to do this after we escape html tags
    if let Some((state, chat, visual)) = data {
        if let Some(participants) = state
            .get_chat_by_id(*chat)
            .map(|c| state.chat_participants(&c))
        {
            let (line, _) = parse_mentions(&text, &participants, &state.did_key(), false, |id| {
                mention_replacement_pattern(id, visual)
            });
            text = line;
        }
    }
    if should_markdown {
        markdown(&text, emojis)
    } else if emojis {
        let s = replace_emojis(text.trim());
        if is_only_emojis(&s) {
            format!("<span class=\"big-emoji\">{s}</span>")
        } else {
            format!("<p>{s}</p>")
        }
    } else {
        format!("<p>{}</p>", text.trim())
    }
}

pub(crate) fn stack_processor(stack: &str, unescape_html: bool, emojis: bool) -> &str {
    if unescape_html {
        if let Some((esc, _)) = HTML_ESCAPES.iter().find(|(_, s)| stack.eq(*s)) {
            return esc;
        }
        if "&nbsp;".eq(stack) {
            return " ";
        }
    }
    if !emojis {
        return stack;
    }
    match stack {
        "<3" => "❤️",
        ">:)" => "😈",
        ">:(" => "😠",
        ":)" => "🙂",
        ":(" => " 🙁",
        ":/" => "🫤",
        ";)" => "😉",
        ":D" => "😁",
        "xD" => "😆",
        ":p" | ":P" => "😛",
        ";p" | ";P" => "😜",
        "xP" => "😝",
        ":|" => "😐",
        ":O" => "😮",
        _ => stack,
    }
}

pub fn process_string<F>(input: &str, processor: F) -> String
where
    F: Fn(&str) -> &str,
{
    let mut builder = String::new();
    let mut stack = String::new();

    for char in input.chars() {
        match char {
            ' ' => {
                builder += processor(&stack);
                stack.clear();
                builder.push(char);
            }
            _ => stack.push(char),
        }
    }

    builder += processor(&stack);
    builder
}

pub fn replace_emojis(input: &str) -> String {
    process_string(input, |s| stack_processor(s, false, true))
}

struct RegexReplacer;

impl Replacer for RegexReplacer {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        dst.push_str(&caps[1]);
        if caps[2].eq(" ") {
            dst.push_str("&nbsp;");
        } else {
            dst.push_str(&caps[2].replace("&gt;", ">"));
        }
    }
}

pub(crate) fn markdown(text: &str, emojis: bool) -> String {
    let txt = text.trim();
    if emojis {
        let r = replace_emojis(txt);
        // TODO: Watch this issue for a fix: https://github.com/open-i18n/rust-unic/issues/280
        // This is a temporary workaround for some characters unic-emoji-char thinks are emojis
        if !r.chars().all(char::is_alphanumeric) // for any numbers, eg 1, 11, 111
           && r != "#"
           && r != "*"
           && r != "##"
           && r != "**"
           && r != "-"
           && is_only_emojis(&r)
        {
            return format!("<span class=\"big-emoji\">{r}</span>");
        } else if is_only_emojis(txt) || r == "-" {
            return format!("<p>{txt}</p>");
        }
    }

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let text = MARKDOWN_PROCESSOR_REGEX.replace_all(txt, RegexReplacer);

    let mut html_output = String::new();
    let mut in_paragraph = false;
    let mut in_code_block = false;
    let (mut skipping, mut in_link) = (false, false);

    let parser = pulldown_cmark::Parser::new_ext(&text, options);
    for (event, range) in parser.into_offset_iter() {
        if skipping {
            skipping = if in_link {
                matches!(event, pulldown_cmark::Event::End(TagEnd::Link))
            } else {
                matches!(event, pulldown_cmark::Event::End(TagEnd::Image))
            };
            continue;
        }
        match event {
            pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(
                CodeBlockKind::Indented,
            )) => {
                html_output.push_str("</p>\n<p> </p><p>");
            }
            pulldown_cmark::Event::Code(mut txt) => {
                txt = HTML_ESCAPES
                    .iter()
                    .fold(txt, |s, (to, from)| s.replace(*from, to).into());
                pulldown_cmark::html::push_html(
                    &mut html_output,
                    std::iter::once(pulldown_cmark::Event::Code(txt)),
                )
            }
            pulldown_cmark::Event::End(TagEnd::CodeBlock) => {}
            pulldown_cmark::Event::SoftBreak => {
                if in_paragraph {
                    html_output.push_str("</p>\n<p>");
                }
            }
            pulldown_cmark::Event::Start(Tag::Paragraph) => {
                in_paragraph = true;
                html_output.push_str("<p>");
            }
            pulldown_cmark::Event::End(TagEnd::Paragraph) => {
                in_paragraph = false;
            }
            pulldown_cmark::Event::Start(Tag::Image { .. })
            | pulldown_cmark::Event::Start(Tag::Link { .. }) => {
                // Ignore links and image parsing
                // We only want Autolink but that doesn't work (or needs <> which we also dont weed)
                skipping = true;
                in_link = matches!(event, pulldown_cmark::Event::End(TagEnd::Link));
                html_output.push_str(&text[range]);
            }
            pulldown_cmark::Event::Text(t) => {
                let text = if emojis || in_code_block {
                    process_string(&t, |s| stack_processor(s, in_code_block, emojis))
                } else {
                    t.to_string()
                };
                let txt: pulldown_cmark::CowStr<'_> = if in_paragraph {
                    text.replace("\n\n", "<br/>").into()
                } else {
                    text.into()
                };
                if in_code_block {
                    html_output.push_str(&txt);
                } else {
                    pulldown_cmark::html::push_html(
                        &mut html_output,
                        std::iter::once(pulldown_cmark::Event::Text(txt)),
                    );
                }
            }
            event => {
                match event {
                    pulldown_cmark::Event::Start(Tag::CodeBlock(_)) => {
                        in_code_block = true;
                    }
                    pulldown_cmark::Event::End(TagEnd::CodeBlock) => {
                        in_code_block = false;
                    }
                    _ => {}
                }
                pulldown_cmark::html::push_html(&mut html_output, std::iter::once(event))
            }
        }
    }
    html_output.push('\n');
    html_output
}

use unic_emoji_char::{
    is_emoji, is_emoji_component, is_emoji_modifier, is_emoji_modifier_base, is_emoji_presentation,
};

// matches strings consisting of emojis and whitespace
pub fn is_only_emojis(input: &str) -> bool {
    let input = input.trim();
    if emojis::get(input).is_some() {
        return true;
    }
    let mut indices = unic_segment::GraphemeIndices::new(input);
    indices.all(|(_, grapheme)| {
        grapheme.trim().chars().all(|c| {
            is_emoji(c)
            || is_emoji_component(c)
            || is_emoji_modifier(c)
            || is_emoji_modifier_base(c)
            || is_emoji_presentation(c)
            // some emojis are multiple emojis joined by this character
            || c == '\u{200d}'
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ wrap_links_with_a_tags tests ============

    #[test]
    fn link_replacer_wraps_http_url() {
        let text = "visit https://example.com please";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href=\"https://example.com\">https://example.com</a>"));
        assert_eq!(links, vec!["https://example.com"]);
    }

    #[test]
    fn link_replacer_wraps_https_url() {
        let text = "Check https://secure.example.org out";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(
            html.contains("<a href=\"https://secure.example.org\">https://secure.example.org</a>")
        );
        assert_eq!(links, vec!["https://secure.example.org"]);
    }

    #[test]
    fn link_replacer_recognizes_tld_other_than_com() {
        let text = "visit https://example.faketld please";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href=\"https://example.faketld\">https://example.faketld</a>"));
        assert_eq!(links, vec!["https://example.faketld"]);
    }

    #[test]
    fn link_replacer_adds_https_to_www() {
        let text = "Go to www.example.com now";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href=\"https://www.example.com\">www.example.com</a>"));
        assert_eq!(links, vec!["https://www.example.com"]);
    }

    #[test]
    fn link_replacer_handles_multiple_urls() {
        let text = "Visit https://example1.com and www.example2.org";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("https://example1.com"));
        assert!(html.contains("https://www.example2.org"));
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn link_replacer_handles_mailto_links() {
        let text = "Email me at mailto: user@example.com";
        let (html, _links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href=\"mailto: user@example.com\">user@example.com</a>"));
    }

    #[test]
    fn link_replacer_no_link_unchanged() {
        let text = "No url in this message";
        let (html, links) = wrap_links_with_a_tags(text);
        assert_eq!(html, text);
        assert!(links.is_empty());
    }

    #[test]
    fn link_replacer_with_url_containing_path() {
        let text = "Check https://example.com/path/to/page";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("https://example.com/path/to/page"));
        assert_eq!(links, vec!["https://example.com/path/to/page"]);
    }

    #[test]
    fn link_replacer_with_url_containing_parentheses() {
        let text = "See (https://example.com/page)";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href"));
        assert!(!links.is_empty());
    }

    // ============ replace_emojis tests ============

    #[test]
    fn replace_emojis_smiley() {
        let result = replace_emojis("Hello :) friend");
        assert!(result.contains("🙂"));
    }

    #[test]
    fn replace_emojis_sad_face() {
        let result = replace_emojis("I'm sad :(");
        assert!(result.contains("🙁"));
    }

    #[test]
    fn replace_emojis_wink() {
        let result = replace_emojis("Just kidding ;)");
        assert!(result.contains("😉"));
    }

    #[test]
    fn replace_emojis_big_smile() {
        let result = replace_emojis("Very happy :D");
        assert!(result.contains("😁"));
    }

    #[test]
    fn replace_emojis_evil_smile() {
        let result = replace_emojis(">:) muahahaha");
        assert!(result.contains("😈"));
    }

    #[test]
    fn replace_emojis_heart() {
        let result = replace_emojis("I love you <3");
        assert!(result.contains("❤️"));
    }

    #[test]
    fn replace_emojis_no_emoji() {
        let result = replace_emojis("Plain text");
        assert_eq!(result, "Plain text");
    }

    #[test]
    fn replace_emojis_multiple() {
        let result = replace_emojis(":) and ;) but :(");
        assert!(result.contains("🙂"));
        assert!(result.contains("😉"));
        assert!(result.contains("🙁"));
    }

    #[test]
    fn replace_emojis_neutral_face() {
        let result = replace_emojis("I'm neutral :/");
        assert!(result.contains("🫤"));
    }

    #[test]
    fn replace_emojis_tongue_out() {
        let result = replace_emojis("Silly :p");
        assert!(result.contains("😛"));
    }

    #[test]
    fn replace_emojis_xd() {
        let result = replace_emojis("Very funny xD");
        assert!(result.contains("😆"));
    }

    #[test]
    fn replace_emojis_evil_face_variant() {
        let result = replace_emojis("Evil >:(");
        assert!(result.contains("😠"));
    }

    #[test]
    fn replace_emojis_expressionless() {
        let result = replace_emojis("Nothing to say :|");
        assert!(result.contains("😐"));
    }

    #[test]
    fn replace_emojis_surprised() {
        let result = replace_emojis("What :O");
        assert!(result.contains("😮"));
    }

    // ============ is_only_emojis tests ============

    #[test]
    fn is_only_emojis_single_emoji() {
        assert!(is_only_emojis("😀"));
    }

    #[test]
    fn is_only_emojis_multiple_emojis() {
        assert!(is_only_emojis("😀😁😂"));
    }

    #[test]
    fn is_only_emojis_with_whitespace() {
        assert!(is_only_emojis("  😀 😁  "));
    }

    #[test]
    fn is_only_emojis_text_and_emoji() {
        assert!(!is_only_emojis("Hello 😀"));
    }

    #[test]
    fn is_only_emojis_empty_string() {
        assert!(is_only_emojis(""));
    }

    #[test]
    fn is_only_emojis_plain_text() {
        assert!(!is_only_emojis("hello world"));
    }

    #[test]
    fn is_only_emojis_emoji_with_zwj() {
        assert!(is_only_emojis("👨‍👩‍👧‍👦"));
    }

    #[test]
    fn is_only_emojis_with_special_chars() {
        assert!(!is_only_emojis("😀!@#"));
    }

    // ============ process_string tests ============

    #[test]
    fn process_string_basic() {
        let result = process_string("hello world", |s| s);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn process_string_empty() {
        let result = process_string("", |s| s);
        assert_eq!(result, "");
    }

    #[test]
    fn process_string_single_word() {
        let result = process_string("hello", |s| s);
        assert_eq!(result, "hello");
    }

    #[test]
    fn process_string_with_callback() {
        let result = process_string("a b c", |s| if s == "b" { "B" } else { s });
        assert!(result.contains("B"));
    }

    #[test]
    fn process_string_with_special_characters() {
        let result = process_string("hello@world#test", |s| s);
        assert!(result.contains("@"));
        assert!(result.contains("#"));
    }

    // ============ stack_processor tests ============

    #[test]
    fn stack_processor_emoji_smiley() {
        let result = stack_processor(":)", false, true);
        assert_eq!(result, "🙂");
    }

    #[test]
    fn stack_processor_emoji_heart() {
        let result = stack_processor("<3", false, true);
        assert_eq!(result, "❤️");
    }

    #[test]
    fn stack_processor_no_emoji_mode() {
        let result = stack_processor(":)", false, false);
        assert_eq!(result, ":)");
    }

    #[test]
    fn stack_processor_unescape_html() {
        let result = stack_processor("&amp;", true, false);
        assert_eq!(result, "&");
    }

    #[test]
    fn stack_processor_unescape_nbsp() {
        let result = stack_processor("&nbsp;", true, false);
        assert_eq!(result, " ");
    }

    #[test]
    fn stack_processor_unknown_input() {
        let result = stack_processor("xyz", false, true);
        assert_eq!(result, "xyz");
    }

    #[test]
    fn stack_processor_evil_face() {
        let result = stack_processor(">:(", false, true);
        assert_eq!(result, "😠");
    }

    #[test]
    fn stack_processor_tongue_wink() {
        let result = stack_processor(";p", false, true);
        assert_eq!(result, "😜");
    }

    #[test]
    fn stack_processor_neutral() {
        let result = stack_processor(":/", false, true);
        assert_eq!(result, "🫤");
    }

    #[test]
    fn stack_processor_expressionless() {
        let result = stack_processor(":|", false, true);
        assert_eq!(result, "😐");
    }

    #[test]
    fn stack_processor_surprised() {
        let result = stack_processor(":O", false, true);
        assert_eq!(result, "😮");
    }

    // ============ markdown tests ============

    #[test]
    fn markdown_plain_text() {
        let result = markdown("hello world", false);
        assert!(result.contains("hello world"));
    }

    #[test]
    fn markdown_with_emojis() {
        let result = markdown("hello :)", true);
        assert!(result.contains("🙂"));
    }

    #[test]
    fn markdown_bold() {
        let result = markdown("**bold text**", false);
        assert!(result.contains("<strong>"));
    }

    #[test]
    fn markdown_italic() {
        let result = markdown("*italic text*", false);
        assert!(result.contains("<em>"));
    }

    #[test]
    fn markdown_strikethrough() {
        let result = markdown("~~strikethrough~~", false);
        assert!(result.contains("<del>"));
    }

    #[test]
    fn markdown_code_inline() {
        let result = markdown("`code`", false);
        assert!(result.contains("<code>"));
    }

    #[test]
    fn markdown_empty_string() {
        let result = markdown("", false);
        assert!(!result.is_empty());
    }

    #[test]
    fn markdown_with_newlines() {
        let result = markdown("line1\nline2", false);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn markdown_only_emojis() {
        let result = markdown("😀😁", true);
        assert!(result.contains("big-emoji"));
    }

    #[test]
    fn markdown_ignores_links() {
        let result = markdown("[link](https://example.com)", false);
        assert!(!result.contains("href"));
    }

    #[test]
    fn markdown_with_special_characters() {
        let result = markdown("Text with &, <, > chars", false);
        assert!(result.contains("&amp;"));
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
    }

    #[test]
    fn markdown_with_list() {
        let result = markdown("- item 1\n- item 2", false);
        assert!(result.contains("<li>"));
    }

    #[test]
    fn markdown_code_block() {
        let result = markdown("```\ncode block\n```", false);
        assert!(result.contains("code"));
    }

    // ============ format_text tests ============

    #[test]
    fn format_text_plain_no_markdown_no_emoji() {
        let result = format_text("hello world", false, false, None);
        assert_eq!(result, "<p>hello world</p>");
    }

    #[test]
    fn format_text_with_markdown_enabled() {
        let result = format_text("**bold**", true, false, None);
        assert!(result.contains("<strong>"));
    }

    #[test]
    fn format_text_with_emoji_enabled() {
        let result = format_text("hello :)", false, true, None);
        assert!(result.contains("🙂"));
    }

    #[test]
    fn format_text_with_markdown_and_emoji() {
        let result = format_text("**hello** :)", true, true, None);
        assert!(result.contains("<strong>"));
        assert!(result.contains("🙂"));
    }

    #[test]
    fn format_text_empty_string() {
        let result = format_text("", false, false, None);
        assert!(result.contains("<p>"));
    }

    #[test]
    fn format_text_html_escape_ampersand() {
        let result = format_text("Tom & Jerry", false, false, None);
        assert!(result.contains("&amp;"));
    }

    #[test]
    fn format_text_html_escape_less_than() {
        let result = format_text("5 < 10", false, false, None);
        assert!(result.contains("&lt;"));
    }

    #[test]
    fn format_text_html_escape_greater_than() {
        let result = format_text("10 > 5", false, false, None);
        assert!(result.contains("&gt;"));
    }

    #[test]
    fn format_text_html_escape_quote() {
        let result = format_text("He said \"hi\"", false, false, None);
        assert!(result.contains("&quot;"));
    }

    #[test]
    fn format_text_html_escape_apostrophe() {
        let result = format_text("Don't", false, false, None);
        assert!(result.contains("&#x27;"));
    }

    #[test]
    fn format_text_only_emojis() {
        let result = format_text("😀", false, true, None);
        assert!(result.contains("big-emoji"));
    }

    #[test]
    fn format_text_preserves_newlines_as_nbsp() {
        let result = format_text("line1\nline2", false, false, None);
        assert!(result.contains("&nbsp;&nbsp;"));
    }

    #[test]
    fn format_text_markdown_disabled_emoji_enabled() {
        let result = format_text("**not bold** :)", false, true, None);
        assert!(!result.contains("<strong>"));
        assert!(result.contains("🙂"));
    }

    #[test]
    fn format_text_markdown_enabled_emoji_disabled() {
        let result = format_text("**bold** :)", true, false, None);
        assert!(result.contains("<strong>"));
        assert!(!result.contains("🙂"));
    }

    #[test]
    fn format_text_xss_prevention() {
        let result = format_text("<script>alert('xss')</script>", false, false, None);
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(!result.contains("<script>"));
    }

    // ============ Order enum tests ============

    #[test]
    fn order_first_display() {
        let order = Order::First;
        assert_eq!(order.to_string(), "message-first");
    }

    #[test]
    fn order_middle_display() {
        let order = Order::Middle;
        assert_eq!(order.to_string(), "message-middle");
    }

    #[test]
    fn order_last_display() {
        let order = Order::Last;
        assert_eq!(order.to_string(), "message-last");
    }

    // ============ ReactionAdapter tests ============

    #[test]
    fn reaction_adapter_creation() {
        let reaction = ReactionAdapter {
            emoji: "😀".to_string(),
            alt: "grinning".to_string(),
            self_reacted: true,
            reaction_count: 5,
        };
        assert_eq!(reaction.emoji, "😀");
        assert_eq!(reaction.reaction_count, 5);
        assert!(reaction.self_reacted);
    }

    // ============ HTML_ESCAPES tests ============

    #[test]
    fn html_escapes_ampersand() {
        let (from, to) = HTML_ESCAPES[0];
        assert_eq!(from, "&");
        assert_eq!(to, "&amp;");
    }

    #[test]
    fn html_escapes_less_than() {
        let (from, to) = HTML_ESCAPES[1];
        assert_eq!(from, "<");
        assert_eq!(to, "&lt;");
    }

    #[test]
    fn html_escapes_greater_than() {
        let (from, to) = HTML_ESCAPES[2];
        assert_eq!(from, ">");
        assert_eq!(to, "&gt;");
    }

    #[test]
    fn html_escapes_quote() {
        let (from, to) = HTML_ESCAPES[3];
        assert_eq!(from, "\"");
        assert_eq!(to, "&quot;");
    }

    #[test]
    fn html_escapes_apostrophe() {
        let (from, to) = HTML_ESCAPES[4];
        assert_eq!(from, "'");
        assert_eq!(to, "&#x27;");
    }

    // ============ Integration tests ============

    #[test]
    fn integration_markdown_with_links_and_emojis() {
        let text = "Check **this** link: https://example.com :)";
        let result = format_text(text, true, true, None);
        assert!(result.contains("<strong>"));
        assert!(result.contains("https://example.com"));
        assert!(result.contains("🙂"));
    }

    #[test]
    fn integration_html_escape_then_markdown() {
        let text = "This <script> tag & markdown **bold**";
        let result = format_text(text, true, false, None);
        assert!(result.contains("&lt;"));
        assert!(result.contains("&amp;"));
        assert!(result.contains("<strong>"));
    }

    #[test]
    fn integration_only_emojis_detection() {
        let text = "😀😁😂";
        let result = format_text(text, false, true, None);
        assert!(result.contains("big-emoji"));
    }

    #[test]
    fn integration_mixed_emojis_text() {
        let text = "Hello :) world";
        let result = format_text(text, false, true, None);
        assert!(result.contains("Hello"));
        assert!(result.contains("🙂"));
        assert!(result.contains("world"));
    }

    #[test]
    fn integration_complex_markdown() {
        let text = "**bold** *italic* ~~strikethrough~~ `code`";
        let result = format_text(text, true, false, None);
        assert!(result.contains("<strong>"));
        assert!(result.contains("<em>"));
        assert!(result.contains("<del>"));
        assert!(result.contains("<code>"));
    }
}
