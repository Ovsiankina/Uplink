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

    /// Utilise l'entiereté le module supérieur, dans ce cas là, c'est
    /// ce fichier-ci en entier. (mod.rs en Rust est la façon dont
    /// on déclare un module, similaire aux `namespace` en C++)
    use super::*;

    // --- replace_append (via wrap_links_with_a_tags) ---

    /// Un lien simple http doit être enveloppé dans une balise <a>.
    #[test]
    fn link_replacer_wraps_http_url() {
        let text = "visit https://example.com please";
        let (html, links) = wrap_links_with_a_tags(text);

        assert!(html.contains("<a href=\"https://example.com\">https://example.com</a>"));
        assert_eq!(links, vec!["https://example.com"]);
    }

    #[test]
    fn link_replacer_does_reconize_tld_other_than_dot_com() {
        let text = "visit https://example.faketld please";
        let (html, links) = wrap_links_with_a_tags(text);

        assert!(html.contains("<a href=\"https://example.faketld\">https://example.faketld</a>"));
        assert_eq!(links, vec!["https://example.faketld"]);
    }

    /// Un lien www. sans schéma doit recevoir le préfixe https://.
    #[test]
    fn link_replacer_adds_https_to_www() {
        let text = "Dépéche toi d'aller à www.example.com MAINTENANT";
        let (html, links) = wrap_links_with_a_tags(text);

        assert!(html.contains("<a href=\"https://www.example.com\">www.example.com</a>"));
        assert_eq!(links, vec!["https://www.example.com"]);
    }

    /// Un texte sans lien ne doit pas être modifié.
    #[test]
    fn link_replacer_no_link_unchanged() {
        let text = "Pas d'url dans ce message";
        let (html, links) = wrap_links_with_a_tags(text);

        assert_eq!(html, text);
        assert!(links.is_empty());
    }

    /// Un texte sans .com
    #[test]
    fn link_replacer_no_com_unchanged() {
        let text = "texte ennuyant du pote qu'on voit jamais";
        let (html, links) = wrap_links_with_a_tags(text);

        assert_eq!(html, text);
        assert!(links.is_empty());
    }

    /// Le regex ne détecte pas les domaines nus sans préfixe
    /// Seuls les URLs avec "www." ou "http(s)://" sont reconnus.
    #[test]
    fn link_replacer_with_no_domain_not_detected() {
        let text = "Regardez mon super site !  example.com";
        let (html, links) = wrap_links_with_a_tags(text);

        // Comportement attendu : aucun lien détecté, texte inchangé
        assert_eq!(html, text);
        assert!(links.is_empty());
    }

    /// Le texte normal n'est pas modifié
    #[test]
    fn format_text_plain_no_markdown_no_emoji() {
        let result = format_text("hello world", false, false, None);
        assert_eq!(result, "<p>hello world</p>");
    }

    // #[test]
    // fn format_text_plain_no_markdown_no_emoji() {
    //     let should_markdown = true;
    //     let emojis = false;
    //
    //     let result = format_text("hello world", should_markdown, emojis, None);
    //     assert_eq!(result, "<p>hello world</p>");
    // }
}
