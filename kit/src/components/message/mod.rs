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

    /// Vérifie qu'une URL absolue présente dans un texte libre est détectée
    /// par la regex et enveloppée dans une balise `<a href="...">...</a>`,
    /// avec un attribut `href` identique au texte affiché. L'URL doit
    /// également être remontée dans le vecteur `links` retourné par la
    /// fonction, afin que l'appelant puisse la traiter ultérieurement
    /// (par exemple pour ouvrir le lien dans un navigateur externe).
    #[test]
    fn link_replacer_wraps_http_url() {
        let text = "visit https://example.com please";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href=\"https://example.com\">https://example.com</a>"));
        assert_eq!(links, vec!["https://example.com"]);
    }

    /// Variante du cas précédent avec un domaine différent
    /// (`secure.example.org`). Confirme que la détection ne dépend pas du
    /// nom de domaine particulier mais bien de la structure de l'URL :
    /// schéma `https://`, suivi d'un domaine et d'un TLD.
    #[test]
    fn link_replacer_wraps_https_url() {
        let text = "Check https://secure.example.org out";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(
            html.contains("<a href=\"https://secure.example.org\">https://secure.example.org</a>")
        );
        assert_eq!(links, vec!["https://secure.example.org"]);
    }

    /// Vérifie que la regex accepte les TLD autres que les classiques
    /// `.com` / `.org` / `.net`. Ce test utilise `.faketld` comme TLD
    /// arbitraire pour s'assurer que la détection n'est pas verrouillée
    /// sur une liste fermée — important pour la prise en charge des
    /// nouveaux TLD régulièrement enregistrés (`.dev`, `.app`, `.xyz`...).
    #[test]
    fn link_replacer_recognizes_tld_other_than_com() {
        let text = "visit https://example.faketld please";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href=\"https://example.faketld\">https://example.faketld</a>"));
        assert_eq!(links, vec!["https://example.faketld"]);
    }

    /// Vérifie qu'une URL commençant par `www.` (sans schéma explicite) se
    /// voit automatiquement préfixer un `https://` dans l'attribut `href`,
    /// tout en conservant la forme originale `www.example.com` dans le
    /// texte affiché à l'utilisateur. Cette normalisation garantit que les
    /// liens restent cliquables sans rendre le rendu visuel inhabituel.
    #[test]
    fn link_replacer_adds_https_to_www() {
        let text = "Go to www.example.com now";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href=\"https://www.example.com\">www.example.com</a>"));
        assert_eq!(links, vec!["https://www.example.com"]);
    }

    /// Vérifie que plusieurs URLs présentes dans un même message sont
    /// détectées de manière indépendante, indépendamment de leur format
    /// (schéma `https://` explicite vs préfixe `www.`). Le vecteur `links`
    /// retourné doit contenir chaque URL distinctement dans l'ordre
    /// d'apparition, sans doublons ni concaténations.
    #[test]
    fn link_replacer_handles_multiple_urls() {
        let text = "Visit https://example1.com and www.example2.org";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("https://example1.com"));
        assert!(html.contains("https://www.example2.org"));
        assert_eq!(links.len(), 2);
    }

    /// Vérifie le traitement des liens `mailto:`. Le texte affiché à
    /// l'utilisateur doit présenter uniquement l'adresse email (sans le
    /// préfixe `mailto:`), tandis que l'attribut `href` conserve le
    /// préfixe pour que le clic déclenche correctement l'ouverture du
    /// client de messagerie système.
    #[test]
    fn link_replacer_handles_mailto_links() {
        let text = "Email me at mailto: user@example.com";
        let (html, _links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href=\"mailto: user@example.com\">user@example.com</a>"));
    }

    /// Cas nominal négatif : un texte ne contenant aucune URL doit être
    /// retourné rigoureusement inchangé, et le vecteur `links` doit être
    /// vide. Ce test garantit l'absence de faux positifs (mots ressemblant
    /// à des URLs mais sans schéma) et l'idempotence de la fonction sur
    /// les messages textuels classiques.
    #[test]
    fn link_replacer_no_link_unchanged() {
        let text = "No url in this message";
        let (html, links) = wrap_links_with_a_tags(text);
        assert_eq!(html, text);
        assert!(links.is_empty());
    }

    /// Vérifie qu'une URL incluant un chemin (`/path/to/page`) est
    /// capturée dans son intégralité, jusqu'au dernier segment du chemin,
    /// et non tronquée au domaine. Important pour les liens vers des
    /// ressources spécifiques (articles, documents, ancres...).
    #[test]
    fn link_replacer_with_url_containing_path() {
        let text = "Check https://example.com/path/to/page";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("https://example.com/path/to/page"));
        assert_eq!(links, vec!["https://example.com/path/to/page"]);
    }

    /// Cas limite : URL entourée de parenthèses dans le texte source
    /// (forme rédactionnelle courante du type « voir (https://...) »).
    /// La regex doit extraire l'URL en excluant les parenthèses
    /// englobantes, sans quoi le caractère `)` final serait inclus dans
    /// le `href` et casserait le lien.
    #[test]
    fn link_replacer_with_url_containing_parentheses() {
        let text = "See (https://example.com/page)";
        let (html, links) = wrap_links_with_a_tags(text);
        assert!(html.contains("<a href"));
        assert!(!links.is_empty());
    }

    // ============ replace_emojis tests ============

    /// Vérifie la conversion de l'émoticône ASCII `:)` en l'emoji Unicode
    /// correspondant 🙂 (U+1F642) lorsqu'elle apparaît au sein d'un texte
    /// libre. Le reste du message doit rester intact, seule la séquence
    /// `:)` étant substituée.
    #[test]
    fn replace_emojis_smiley() {
        let result = replace_emojis("Hello :) friend");
        assert!(result.contains("🙂"));
    }

    /// Symétrique du smiley positif : l'émoticône `:(` doit être convertie
    /// en 🙁 (U+1F641). Vérifie la couverture des émotions négatives de
    /// base, attendues par l'utilisateur final dans une messagerie.
    #[test]
    fn replace_emojis_sad_face() {
        let result = replace_emojis("I'm sad :(");
        assert!(result.contains("🙁"));
    }

    /// Vérifie que le clin d'œil `;)` est converti en 😉 (U+1F609).
    /// Cas légèrement différent du smiley standard car le premier caractère
    /// est un point-virgule plutôt qu'un deux-points, ce qui exerce une
    /// branche distincte du dispatcher d'émoticônes.
    #[test]
    fn replace_emojis_wink() {
        let result = replace_emojis("Just kidding ;)");
        assert!(result.contains("😉"));
    }

    /// Vérifie que `:D` est converti en 😁 (U+1F601, beaming face).
    /// Cette séquence est distincte du smiley simple et représente une
    /// émotion plus enthousiaste.
    #[test]
    fn replace_emojis_big_smile() {
        let result = replace_emojis("Very happy :D");
        assert!(result.contains("😁"));
    }

    /// Cas d'une émoticône composée de trois caractères : `>:)` doit être
    /// convertie en 😈 (U+1F608, smiling face with horns). Vérifie que le
    /// parser gère correctement les séquences plus longues que le simple
    /// duo de caractères, sans confusion avec `:)` qu'elle contient.
    #[test]
    fn replace_emojis_evil_smile() {
        let result = replace_emojis(">:) muahahaha");
        assert!(result.contains("😈"));
    }

    /// Vérifie la conversion du symbole textuel `<3` en cœur ❤️ (U+2764
    /// suivi du sélecteur de variation U+FE0F pour un rendu coloré). Le
    /// sélecteur de variation est essentiel pour que l'emoji soit affiché
    /// en couleur plutôt qu'en glyphe noir et blanc.
    #[test]
    fn replace_emojis_heart() {
        let result = replace_emojis("I love you <3");
        assert!(result.contains("❤️"));
    }

    /// Cas nominal négatif : un texte ne contenant aucune émoticône doit
    /// être retourné rigoureusement inchangé. Garantit que la fonction
    /// n'introduit pas de faux positifs ni de modifications inattendues
    /// sur les messages textuels purs.
    #[test]
    fn replace_emojis_no_emoji() {
        let result = replace_emojis("Plain text");
        assert_eq!(result, "Plain text");
    }

    /// Vérifie que plusieurs émoticônes différentes présentes dans un
    /// même message sont toutes converties en parallèle, sans qu'aucune
    /// n'en bloque ou n'en écrase une autre. Ce test exerce la robustesse
    /// du parser face à des séquences successives séparées par du texte.
    #[test]
    fn replace_emojis_multiple() {
        let result = replace_emojis(":) and ;) but :(");
        assert!(result.contains("🙂"));
        assert!(result.contains("😉"));
        assert!(result.contains("🙁"));
    }

    /// Vérifie que `:/` est converti en 🫤 (U+1FAE4, face with diagonal
    /// mouth). Cet emoji représente l'embarras ou l'incertitude et fait
    /// partie des ajouts plus récents au standard Unicode (Emoji 14.0).
    #[test]
    fn replace_emojis_neutral_face() {
        let result = replace_emojis("I'm neutral :/");
        assert!(result.contains("🫤"));
    }

    /// Vérifie que `:p` est converti en 😛 (U+1F61B, face with tongue).
    /// Émoticône en minuscule, utile pour vérifier que le parser distingue
    /// bien les variantes de casse (cf. `xD` traité séparément).
    #[test]
    fn replace_emojis_tongue_out() {
        let result = replace_emojis("Silly :p");
        assert!(result.contains("😛"));
    }

    /// Vérifie que `xD` (mélange casse/lettre) est converti en 😆 (U+1F606,
    /// grinning squinting face). Cas particulier car la séquence ne
    /// commence pas par un caractère de ponctuation, à l'inverse de la
    /// majorité des émoticônes ASCII.
    #[test]
    fn replace_emojis_xd() {
        let result = replace_emojis("Very funny xD");
        assert!(result.contains("😆"));
    }

    /// Variante de l'émoticône maléfique : `>:(` (au lieu de `>:)` pour le
    /// diable souriant) est converti en 😠 (U+1F620, angry face). Vérifie
    /// la distinction entre les deux émoticônes triphasées ne différant
    /// que par leur dernier caractère.
    #[test]
    fn replace_emojis_evil_face_variant() {
        let result = replace_emojis("Evil >:(");
        assert!(result.contains("😠"));
    }

    /// Vérifie que `:|` est converti en 😐 (U+1F610, neutral face).
    /// L'expression neutre se distingue de `:/` (embarras) — deux émotions
    /// proches mais sémantiquement différentes que le parser doit savoir
    /// discriminer.
    #[test]
    fn replace_emojis_expressionless() {
        let result = replace_emojis("Nothing to say :|");
        assert!(result.contains("😐"));
    }

    /// Vérifie que `:O` (avec O majuscule) est converti en 😮 (U+1F62E,
    /// face with open mouth, surprise). Le caractère `O` majuscule
    /// distingue cette émoticône de `:0` ou autres variantes ; le parser
    /// doit faire une correspondance exacte sur ce caractère.
    #[test]
    fn replace_emojis_surprised() {
        let result = replace_emojis("What :O");
        assert!(result.contains("😮"));
    }

    // ============ is_only_emojis tests ============

    /// Cas le plus simple : un message ne contenant qu'un seul emoji est
    /// reconnu comme « only emojis ». Cette détection conditionne ensuite
    /// l'affichage agrandi (classe CSS `big-emoji`) appliqué aux messages
    /// composés exclusivement d'emojis.
    #[test]
    fn is_only_emojis_single_emoji() {
        assert!(is_only_emojis("😀"));
    }

    /// Vérifie que plusieurs emojis collés sans séparateur sont également
    /// reconnus comme un message « only emojis ». Le décompte des
    /// caractères doit traiter chaque emoji comme une unité, et non
    /// chaque codepoint qui le compose.
    #[test]
    fn is_only_emojis_multiple_emojis() {
        assert!(is_only_emojis("😀😁😂"));
    }

    /// Vérifie que les espaces (avant, entre et après les emojis) sont
    /// ignorés par la détection. Un utilisateur qui sépare ses emojis par
    /// des espaces pour la lisibilité doit néanmoins bénéficier du rendu
    /// agrandi big-emoji.
    #[test]
    fn is_only_emojis_with_whitespace() {
        assert!(is_only_emojis("  😀 😁  "));
    }

    /// Cas négatif : la présence de texte autre qu'emojis (ici « Hello »)
    /// doit invalider la détection « only emojis », même si un emoji est
    /// présent. Ce test garde la fonction d'application excessive du
    /// rendu agrandi sur les messages mixtes.
    #[test]
    fn is_only_emojis_text_and_emoji() {
        assert!(!is_only_emojis("Hello 😀"));
    }

    /// Cas limite : une chaîne vide est considérée comme « only emojis ».
    /// Choix sémantique discutable mais cohérent avec l'invariant « aucun
    /// caractère non-emoji présent » ; l'affichage en aval ne produit
    /// rien d'observable pour une chaîne vide.
    #[test]
    fn is_only_emojis_empty_string() {
        assert!(is_only_emojis(""));
    }

    /// Cas négatif explicite : un texte ordinaire sans aucun emoji doit
    /// retourner `false`. Important pour s'assurer qu'aucun caractère
    /// alphanumérique ASCII n'est faussement classifié comme emoji.
    #[test]
    fn is_only_emojis_plain_text() {
        assert!(!is_only_emojis("hello world"));
    }

    /// Cas spécial des emojis composés à l'aide du Zero-Width Joiner
    /// (U+200D), comme la famille 👨‍👩‍👧‍👦 qui combine plusieurs
    /// codepoints distincts. La détection doit considérer la séquence
    /// complète comme un emoji unique, sans être perturbée par les
    /// caractères ZWJ intermédiaires.
    #[test]
    fn is_only_emojis_emoji_with_zwj() {
        assert!(is_only_emojis("👨‍👩‍👧‍👦"));
    }

    /// Cas négatif vérifiant que les caractères de ponctuation et symboles
    /// ASCII (`!`, `@`, `#`) invalident la détection « only emojis » même
    /// lorsqu'ils sont juxtaposés à un emoji légitime.
    #[test]
    fn is_only_emojis_with_special_chars() {
        assert!(!is_only_emojis("😀!@#"));
    }

    // ============ process_string tests ============

    /// Cas nominal de la fonction utilitaire `process_string` : un texte
    /// composé de plusieurs mots séparés par des espaces, traité avec une
    /// callback identité (laisse chaque mot inchangé), doit retourner le
    /// texte d'origine. Sert de baseline pour valider l'absence d'effet
    /// de bord du mécanisme de découpage et de réassemblage.
    #[test]
    fn process_string_basic() {
        let result = process_string("hello world", |s| s);
        assert_eq!(result, "hello world");
    }

    /// Cas limite : une chaîne vide doit produire une chaîne vide en
    /// sortie, sans panique ni allocation parasite. Garantit que la
    /// fonction est utilisable comme étape de pipeline même lorsque
    /// l'entrée précédente n'a rien produit.
    #[test]
    fn process_string_empty() {
        let result = process_string("", |s| s);
        assert_eq!(result, "");
    }

    /// Vérifie le comportement sur une entrée constituée d'un unique mot
    /// (sans espace) : la fonction ne doit pas ajouter de séparateur
    /// parasite en début ou en fin lorsqu'elle réassemble la sortie.
    #[test]
    fn process_string_single_word() {
        let result = process_string("hello", |s| s);
        assert_eq!(result, "hello");
    }

    /// Vérifie qu'une callback non-triviale est effectivement appliquée à
    /// chaque mot du texte. Ici, le mot « b » isolé est transformé en
    /// « B » majuscule, ce qui prouve que la callback reçoit bien chaque
    /// mot individuellement et que sa valeur de retour est intégrée au
    /// résultat final.
    #[test]
    fn process_string_with_callback() {
        let result = process_string("a b c", |s| if s == "b" { "B" } else { s });
        assert!(result.contains("B"));
    }

    /// Vérifie que les caractères spéciaux non-alphabétiques (`@`, `#`)
    /// sont préservés lorsqu'ils font partie d'un « mot » au sens du
    /// découpeur (séparateur d'espaces). Important car ces caractères
    /// peuvent apparaître dans des identifiants utilisateur ou des
    /// hashtags qui ne doivent pas être altérés.
    #[test]
    fn process_string_with_special_characters() {
        let result = process_string("hello@world#test", |s| s);
        assert!(result.contains("@"));
        assert!(result.contains("#"));
    }

    // ============ stack_processor tests ============

    /// Vérifie le mappage direct d'un token `:)` vers son emoji 🙂
    /// lorsque le mode emoji est activé (`emojis = true`). À la
    /// différence de `replace_emojis`, `stack_processor` opère sur un
    /// token isolé déjà extrait et retourne uniquement la valeur
    /// transformée, sans contexte textuel.
    #[test]
    fn stack_processor_emoji_smiley() {
        let result = stack_processor(":)", false, true);
        assert_eq!(result, "🙂");
    }

    /// Vérifie le mappage du token textuel `<3` vers le cœur ❤️ en mode
    /// emoji. Cas particulier car `<3` contient un caractère HTML spécial
    /// (`<`), exerçant la cohabitation entre conversion d'emoji et
    /// échappement HTML géré par d'autres étapes du pipeline.
    #[test]
    fn stack_processor_emoji_heart() {
        let result = stack_processor("<3", false, true);
        assert_eq!(result, "❤️");
    }

    /// Vérifie qu'avec le mode emoji désactivé, un token reconnu comme
    /// émoticône est retourné inchangé. Permet aux utilisateurs qui
    /// préfèrent lire les émoticônes ASCII (préférence d'accessibilité ou
    /// d'écriture rapide) de désactiver la conversion globalement.
    #[test]
    fn stack_processor_no_emoji_mode() {
        let result = stack_processor(":)", false, false);
        assert_eq!(result, ":)");
    }

    /// Vérifie le mode `unescape_html` : l'entité HTML `&amp;` doit être
    /// retransformée en son caractère d'origine `&`. Mode utilisé par le
    /// pipeline pour annuler un échappement antérieur lorsque le contenu
    /// transite par une étape produisant des entités HTML mais que la
    /// sortie attendue est du texte brut.
    #[test]
    fn stack_processor_unescape_html() {
        let result = stack_processor("&amp;", true, false);
        assert_eq!(result, "&");
    }

    /// Variante du mode `unescape_html` : l'entité `&nbsp;` (espace
    /// insécable) est convertie en espace standard. Important car le
    /// pipeline insère des `&nbsp;` pour préserver les sauts de ligne et
    /// la mise en forme, mais ces entités doivent disparaître dans
    /// certains contextes de traitement.
    #[test]
    fn stack_processor_unescape_nbsp() {
        let result = stack_processor("&nbsp;", true, false);
        assert_eq!(result, " ");
    }

    /// Cas par défaut : un token n'appartenant à aucune table de
    /// correspondance (ici `xyz`, ni emoji ni entité HTML) doit être
    /// retourné inchangé. Garantit que le processor est non-destructif
    /// pour le contenu utilisateur arbitraire.
    #[test]
    fn stack_processor_unknown_input() {
        let result = stack_processor("xyz", false, true);
        assert_eq!(result, "xyz");
    }

    /// Mappage du token `>:(` vers 😠 (visage en colère). Token
    /// triphasé qu'il faut distinguer de `>:)` (diable souriant) — le
    /// processor doit faire correspondre l'intégralité du token, pas
    /// uniquement les deux premiers caractères.
    #[test]
    fn stack_processor_evil_face() {
        let result = stack_processor(">:(", false, true);
        assert_eq!(result, "😠");
    }

    /// Mappage de `;p` (tirer la langue avec clin d'œil) vers 😜.
    /// Variante de `:p` : le point-virgule remplace le deux-points,
    /// produisant un emoji distinct combinant les deux significations.
    #[test]
    fn stack_processor_tongue_wink() {
        let result = stack_processor(";p", false, true);
        assert_eq!(result, "😜");
    }

    /// Mappage de `:/` vers 🫤 (visage à la bouche en biais). Cas
    /// commun de l'expression d'embarras, ajouté tardivement au standard
    /// Unicode (Emoji 14.0).
    #[test]
    fn stack_processor_neutral() {
        let result = stack_processor(":/", false, true);
        assert_eq!(result, "🫤");
    }

    /// Mappage de `:|` vers 😐 (visage neutre). À distinguer de `:/`
    /// (embarras) bien que les deux soient des expressions « neutres »
    /// : le processor doit faire la différence sur le second caractère.
    #[test]
    fn stack_processor_expressionless() {
        let result = stack_processor(":|", false, true);
        assert_eq!(result, "😐");
    }

    /// Mappage de `:O` (O majuscule) vers 😮 (bouche ouverte de
    /// surprise). Le caractère `O` doit être en majuscule pour cette
    /// correspondance — la version minuscule `:o` est traitée comme un
    /// token différent.
    #[test]
    fn stack_processor_surprised() {
        let result = stack_processor(":O", false, true);
        assert_eq!(result, "😮");
    }

    // ============ markdown tests ============

    /// Cas nominal : un texte sans aucune syntaxe markdown doit traverser
    /// la fonction sans être altéré dans son contenu. Vérifie que le
    /// passage par `pulldown-cmark` n'introduit pas de transformations
    /// indésirables sur du texte ordinaire.
    #[test]
    fn markdown_plain_text() {
        let result = markdown("hello world", false);
        assert!(result.contains("hello world"));
    }

    /// Vérifie que les émoticônes ASCII présentes dans un texte markdown
    /// sont converties en emojis Unicode lorsque le flag `emojis = true`
    /// est activé. Confirme l'orchestration entre le rendu markdown et
    /// la conversion d'emojis dans la même étape de pipeline.
    #[test]
    fn markdown_with_emojis() {
        let result = markdown("hello :)", true);
        assert!(result.contains("🙂"));
    }

    /// Vérifie le rendu de la syntaxe gras `**texte**` en balise HTML
    /// `<strong>`, conformément à la spécification CommonMark.
    #[test]
    fn markdown_bold() {
        let result = markdown("**bold text**", false);
        assert!(result.contains("<strong>"));
    }

    /// Vérifie le rendu de la syntaxe italique `*texte*` en balise HTML
    /// `<em>` (emphasis), distincte de `<i>` qui est purement
    /// présentationnelle.
    #[test]
    fn markdown_italic() {
        let result = markdown("*italic text*", false);
        assert!(result.contains("<em>"));
    }

    /// Vérifie le rendu du texte barré `~~texte~~` en balise HTML
    /// `<del>` (extension GFM — GitHub Flavored Markdown — non incluse
    /// dans CommonMark de base, mais activée dans la configuration de
    /// `pulldown-cmark` utilisée par le projet).
    #[test]
    fn markdown_strikethrough() {
        let result = markdown("~~strikethrough~~", false);
        assert!(result.contains("<del>"));
    }

    /// Vérifie le rendu du code inline délimité par des backticks simples
    /// `` `code` `` en balise HTML `<code>`. À l'intérieur d'une telle
    /// balise, les caractères spéciaux sont échappés mais le texte n'est
    /// pas davantage transformé (pas d'emoji, pas de markdown imbriqué).
    #[test]
    fn markdown_code_inline() {
        let result = markdown("`code`", false);
        assert!(result.contains("<code>"));
    }

    /// Cas limite : une chaîne vide doit produire une sortie HTML non
    /// vide (typiquement un paragraphe vide). Garantit que la fonction
    /// est utilisable dans un pipeline sans nécessiter de garde
    /// préalable contre les chaînes vides côté appelant.
    #[test]
    fn markdown_empty_string() {
        let result = markdown("", false);
        assert!(!result.is_empty());
    }

    /// Vérifie que les sauts de ligne (`\n`) sont préservés dans le
    /// rendu : les deux lignes d'origine doivent rester présentes dans
    /// la sortie HTML, indépendamment de la manière dont
    /// `pulldown-cmark` choisit de les structurer (paragraphes, sauts
    /// de ligne durs, etc.).
    #[test]
    fn markdown_with_newlines() {
        let result = markdown("line1\nline2", false);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    /// Vérifie que les messages composés exclusivement d'emojis (ici
    /// deux emojis Unicode) reçoivent la classe CSS `big-emoji` qui
    /// déclenche l'affichage en taille agrandie côté UI. Confirme
    /// l'intégration de la détection `is_only_emojis` dans le rendu
    /// markdown.
    #[test]
    fn markdown_only_emojis() {
        let result = markdown("😀😁", true);
        assert!(result.contains("big-emoji"));
    }

    /// Vérifie que la syntaxe markdown des liens `[texte](url)` est
    /// volontairement ignorée par cette implémentation : aucune balise
    /// `<a href>` n'est produite. Choix conscient — la détection des
    /// liens est confiée à `wrap_links_with_a_tags` en aval, qui
    /// opère uniformément sur les URLs explicites quel que soit leur
    /// emballage syntaxique.
    #[test]
    fn markdown_ignores_links() {
        let result = markdown("[link](https://example.com)", false);
        assert!(!result.contains("href"));
    }

    /// Vérifie l'échappement HTML des caractères spéciaux (`&`, `<`,
    /// `>`) dans le rendu markdown. Première ligne de défense contre
    /// les injections HTML/XSS : aucun de ces caractères ne doit
    /// apparaître brut dans la sortie.
    #[test]
    fn markdown_with_special_characters() {
        let result = markdown("Text with &, <, > chars", false);
        assert!(result.contains("&amp;"));
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
    }

    /// Vérifie le rendu des listes markdown : les items préfixés par `-`
    /// doivent produire des balises `<li>`. Permet aux utilisateurs de
    /// rédiger des messages structurés type bullet-points.
    #[test]
    fn markdown_with_list() {
        let result = markdown("- item 1\n- item 2", false);
        assert!(result.contains("<li>"));
    }

    /// Vérifie le traitement des blocs de code délimités par triple
    /// backtick (` ``` `) : le contenu du bloc doit être préservé
    /// littéralement, sans application des autres transformations
    /// (markdown imbriqué, conversion d'emoji ASCII, etc.).
    #[test]
    fn markdown_code_block() {
        let result = markdown("```\ncode block\n```", false);
        assert!(result.contains("code"));
    }

    // ============ format_text tests ============

    /// Cas le plus simple du point d'entrée `format_text` : une chaîne
    /// purement textuelle, sans markdown ni emoji activés et sans State,
    /// doit être enveloppée dans un paragraphe HTML `<p>...</p>` sans
    /// transformation supplémentaire. Établit la sortie de référence.
    #[test]
    fn format_text_plain_no_markdown_no_emoji() {
        let result = format_text("hello world", false, false, None);
        assert_eq!(result, "<p>hello world</p>");
    }

    /// Vérifie que l'activation du flag markdown (`true`) déclenche le
    /// traitement par `pulldown-cmark` : la syntaxe `**bold**` doit
    /// produire une balise `<strong>` dans la sortie. Confirme que le
    /// flag est bien propagé au sous-pipeline markdown.
    #[test]
    fn format_text_with_markdown_enabled() {
        let result = format_text("**bold**", true, false, None);
        assert!(result.contains("<strong>"));
    }

    /// Vérifie que l'activation du flag emoji (`true`), même sans
    /// markdown, suffit à déclencher la conversion des émoticônes
    /// ASCII en emojis Unicode. Confirme l'indépendance des deux
    /// flags `markdown` et `emojis`.
    #[test]
    fn format_text_with_emoji_enabled() {
        let result = format_text("hello :)", false, true, None);
        assert!(result.contains("🙂"));
    }

    /// Vérifie le bon fonctionnement simultané des deux transformations
    /// (markdown + emoji) : une syntaxe `**hello**` est rendue en
    /// `<strong>` ET l'émoticône `:)` est convertie en 🙂. Pas
    /// d'interférence entre les deux étapes.
    #[test]
    fn format_text_with_markdown_and_emoji() {
        let result = format_text("**hello** :)", true, true, None);
        assert!(result.contains("<strong>"));
        assert!(result.contains("🙂"));
    }

    /// Cas limite : une chaîne vide doit produire un HTML structurel
    /// minimal (au moins un `<p>` ouvrant) plutôt qu'une chaîne vide.
    /// Garantit que la sortie est toujours un fragment HTML bien formé,
    /// quel que soit le contenu d'entrée.
    #[test]
    fn format_text_empty_string() {
        let result = format_text("", false, false, None);
        assert!(result.contains("<p>"));
    }

    /// Premier test de la suite d'échappement HTML : le caractère `&`
    /// doit être systématiquement converti en l'entité `&amp;` avant
    /// toute autre transformation. Sans cet échappement, des entités
    /// HTML déjà présentes dans le texte seraient à tort interprétées
    /// par le navigateur.
    #[test]
    fn format_text_html_escape_ampersand() {
        let result = format_text("Tom & Jerry", false, false, None);
        assert!(result.contains("&amp;"));
    }

    /// Échappement du caractère `<` en `&lt;`. Cet échappement est
    /// critique pour la sécurité : il empêche tout texte utilisateur
    /// commençant par `<` d'être interprété comme l'ouverture d'une
    /// balise HTML — fondement de la prévention XSS.
    #[test]
    fn format_text_html_escape_less_than() {
        let result = format_text("5 < 10", false, false, None);
        assert!(result.contains("&lt;"));
    }

    /// Échappement du caractère `>` en `&gt;`. Symétrique de `<`,
    /// nécessaire pour empêcher la fermeture inopinée d'une balise et
    /// pour garantir un rendu correct des comparaisons mathématiques
    /// dans les messages.
    #[test]
    fn format_text_html_escape_greater_than() {
        let result = format_text("10 > 5", false, false, None);
        assert!(result.contains("&gt;"));
    }

    /// Échappement du guillemet double `"` en `&quot;`. Important
    /// pour empêcher la rupture des attributs HTML lorsqu'un texte
    /// utilisateur est interpolé dans un attribut (cas exploité dans
    /// les vecteurs XSS basés sur les attributs).
    #[test]
    fn format_text_html_escape_quote() {
        let result = format_text("He said \"hi\"", false, false, None);
        assert!(result.contains("&quot;"));
    }

    /// Échappement de l'apostrophe `'` en `&#x27;` (notation
    /// numérique hexadécimale). Échappement souvent omis par les
    /// implémentations naïves mais nécessaire face aux attributs HTML
    /// quoted en simple-quote, vecteur XSS classique.
    #[test]
    fn format_text_html_escape_apostrophe() {
        let result = format_text("Don't", false, false, None);
        assert!(result.contains("&#x27;"));
    }

    /// Vérifie que lorsque le texte est composé exclusivement d'un ou
    /// plusieurs emojis Unicode, la classe CSS `big-emoji` est appliquée
    /// pour déclencher le rendu agrandi côté UI. Comportement attendu
    /// dans la majorité des messageries modernes.
    #[test]
    fn format_text_only_emojis() {
        let result = format_text("😀", false, true, None);
        assert!(result.contains("big-emoji"));
    }

    /// Vérifie que les sauts de ligne (`\n`) du texte source sont
    /// convertis en deux espaces insécables (`&nbsp;&nbsp;`) dans le
    /// rendu. Choix de design pour préserver les retours visuels même
    /// dans des contextes où `<br>` n'est pas approprié (par exemple
    /// dans des paragraphes pulldown-cmark).
    #[test]
    fn format_text_preserves_newlines_as_nbsp() {
        let result = format_text("line1\nline2", false, false, None);
        assert!(result.contains("&nbsp;&nbsp;"));
    }

    /// Vérifie le découplage complet des flags : avec `markdown = false`
    /// et `emojis = true`, la syntaxe `**not bold**` doit rester
    /// littérale (aucun `<strong>` produit) tandis que `:)` est bien
    /// convertie. Garantit qu'un utilisateur peut désactiver le rendu
    /// markdown sans perdre les emojis.
    #[test]
    fn format_text_markdown_disabled_emoji_enabled() {
        let result = format_text("**not bold** :)", false, true, None);
        assert!(!result.contains("<strong>"));
        assert!(result.contains("🙂"));
    }

    /// Configuration symétrique : `markdown = true`, `emojis = false`.
    /// `**bold**` est rendu en `<strong>` mais `:)` reste littérale,
    /// non convertie en 🙂. Confirme l'orthogonalité totale entre les
    /// deux flags.
    #[test]
    fn format_text_markdown_enabled_emoji_disabled() {
        let result = format_text("**bold** :)", true, false, None);
        assert!(result.contains("<strong>"));
        assert!(!result.contains("🙂"));
    }

    /// Test de prévention XSS au niveau de `format_text` : une tentative
    /// d'injection `<script>alert('xss')</script>` doit être totalement
    /// neutralisée par échappement HTML. Aucune balise script ne doit
    /// apparaître brute dans la sortie. Test fondamental car
    /// `format_text` produit du HTML qui sera ensuite injecté via
    /// `dangerous_inner_html` côté Dioxus — toute défaillance ici
    /// constituerait une vulnérabilité XSS exploitable.
    #[test]
    fn format_text_xss_prevention() {
        let result = format_text("<script>alert('xss')</script>", false, false, None);
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(!result.contains("<script>"));
    }

    // ============ Order enum tests ============

    /// Vérifie que la variante `First` de l'enum `Order` produit la
    /// classe CSS `message-first` via son implémentation `Display`.
    /// Utilisée pour styler le premier message d'un groupe de messages
    /// consécutifs du même auteur (coins arrondis, espacement).
    #[test]
    fn order_first_display() {
        let order = Order::First;
        assert_eq!(order.to_string(), "message-first");
    }

    /// Vérifie que la variante `Middle` produit `message-middle`.
    /// Appliquée aux messages intermédiaires d'un groupe consécutif :
    /// pas d'espacement supplémentaire au-dessus ni en-dessous, coins
    /// non arrondis aux jonctions.
    #[test]
    fn order_middle_display() {
        let order = Order::Middle;
        assert_eq!(order.to_string(), "message-middle");
    }

    /// Vérifie que la variante `Last` produit `message-last`.
    /// Appliquée au dernier message d'un groupe consécutif : espacement
    /// final, coins arrondis bas, et possiblement affichage de l'avatar
    /// ou du timestamp selon la configuration UI.
    #[test]
    fn order_last_display() {
        let order = Order::Last;
        assert_eq!(order.to_string(), "message-last");
    }

    // ============ ReactionAdapter tests ============

    /// Vérifie la construction et l'accessibilité des champs de la
    /// structure `ReactionAdapter`, qui sert de DTO entre l'état Warp
    /// (réactions distantes) et le composant UI. Les quatre champs
    /// (emoji, alt, self_reacted, reaction_count) doivent être
    /// correctement initialisés et lisibles.
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

    /// Vérifie l'entrée 0 de la table `HTML_ESCAPES` : le caractère
    /// brut `&` est associé à l'entité `&amp;`. Cette entrée doit
    /// impérativement être traitée en premier dans tout pipeline
    /// d'échappement, sous peine de double-échapper les entités
    /// déjà produites par les substitutions ultérieures.
    #[test]
    fn html_escapes_ampersand() {
        let (from, to) = HTML_ESCAPES[0];
        assert_eq!(from, "&");
        assert_eq!(to, "&amp;");
    }

    /// Vérifie l'entrée 1 de la table : `<` → `&lt;`. Pivot de la
    /// défense contre l'injection HTML — si cette correspondance
    /// venait à manquer, n'importe quel texte utilisateur pourrait
    /// ouvrir une balise arbitraire.
    #[test]
    fn html_escapes_less_than() {
        let (from, to) = HTML_ESCAPES[1];
        assert_eq!(from, "<");
        assert_eq!(to, "&lt;");
    }

    /// Vérifie l'entrée 2 de la table : `>` → `&gt;`. Symétrique
    /// de `<`, indispensable pour empêcher la fermeture inopinée
    /// d'une balise lorsque `<` est déjà présent dans le contexte
    /// de rendu.
    #[test]
    fn html_escapes_greater_than() {
        let (from, to) = HTML_ESCAPES[2];
        assert_eq!(from, ">");
        assert_eq!(to, "&gt;");
    }

    /// Vérifie l'entrée 3 de la table : `"` → `&quot;`. Critique
    /// pour les contextes d'attribut HTML où une valeur quotée
    /// pourrait être prématurément fermée par un guillemet
    /// utilisateur, ouvrant alors la voie à l'injection d'attributs
    /// arbitraires.
    #[test]
    fn html_escapes_quote() {
        let (from, to) = HTML_ESCAPES[3];
        assert_eq!(from, "\"");
        assert_eq!(to, "&quot;");
    }

    /// Vérifie l'entrée 4 de la table : `'` → `&#x27;` (notation
    /// numérique hexadécimale plutôt que l'entité nommée `&apos;`,
    /// car cette dernière n'est pas universellement reconnue par
    /// tous les contextes HTML — `&#x27;` est strictement portable).
    #[test]
    fn html_escapes_apostrophe() {
        let (from, to) = HTML_ESCAPES[4];
        assert_eq!(from, "'");
        assert_eq!(to, "&#x27;");
    }

    // ============ Integration tests ============

    /// Test d'intégration interne couvrant la combinaison la plus
    /// fréquente en usage réel : un message contenant à la fois du
    /// markdown (gras), un lien explicite et une émoticône ASCII.
    /// Les trois transformations doivent coexister dans une seule
    /// passe de `format_text`, sans qu'aucune n'invalide les autres.
    #[test]
    fn integration_markdown_with_links_and_emojis() {
        let text = "Check **this** link: https://example.com :)";
        let result = format_text(text, true, true, None);
        assert!(result.contains("<strong>"));
        assert!(result.contains("https://example.com"));
        assert!(result.contains("🙂"));
    }

    /// Vérifie l'ordre du pipeline : l'échappement HTML doit avoir
    /// lieu AVANT le rendu markdown. Sans cet ordre, le markdown
    /// pulldown-cmark pourrait à tort interpréter une partie du
    /// `<script>` comme un fragment HTML brut. Avec cet ordre,
    /// `<script>` devient `&lt;script&gt;` puis le markdown
    /// `**bold**` est rendu normalement par-dessus.
    #[test]
    fn integration_html_escape_then_markdown() {
        let text = "This <script> tag & markdown **bold**";
        let result = format_text(text, true, false, None);
        assert!(result.contains("&lt;"));
        assert!(result.contains("&amp;"));
        assert!(result.contains("<strong>"));
    }

    /// Vérifie la détection « only emojis » au niveau de `format_text` :
    /// un message composé de trois emojis Unicode (sans aucun texte)
    /// doit déclencher la classe CSS `big-emoji` dans la sortie. Test
    /// d'intégration entre `replace_emojis`, `is_only_emojis` et le
    /// rendu HTML final.
    #[test]
    fn integration_only_emojis_detection() {
        let text = "😀😁😂";
        let result = format_text(text, false, true, None);
        assert!(result.contains("big-emoji"));
    }

    /// Vérifie qu'un message mêlant texte et émoticône ASCII préserve
    /// les portions textuelles tout en convertissant l'émoticône.
    /// Cas négatif explicite pour la classe `big-emoji` : le mélange
    /// texte+emoji ne doit pas déclencher l'affichage agrandi.
    #[test]
    fn integration_mixed_emojis_text() {
        let text = "Hello :) world";
        let result = format_text(text, false, true, None);
        assert!(result.contains("Hello"));
        assert!(result.contains("🙂"));
        assert!(result.contains("world"));
    }

    /// Vérifie le rendu combiné de toutes les variantes de markdown
    /// supportées en une seule entrée : gras, italique, barré et
    /// code inline. Garantit qu'aucune des transformations
    /// individuelles ne perturbe les autres lorsqu'elles sont
    /// utilisées simultanément, et que pulldown-cmark produit bien
    /// les balises HTML attendues pour chacune.
    #[test]
    fn integration_complex_markdown() {
        let text = "**bold** *italic* ~~strikethrough~~ `code`";
        let result = format_text(text, true, false, None);
        assert!(result.contains("<strong>"));
        assert!(result.contains("<em>"));
        assert!(result.contains("<del>"));
        assert!(result.contains("<code>"));
    }

    // ============ Mention tests (with mocked State) ============

    use std::collections::{HashMap, HashSet, VecDeque};
    use warp::crypto::DID;
    use warp::raygun::ConversationSettings;
    use common::state::{Chat, Chats, Friends, Identity};

    /// Helper: create a minimal State with two identities ("me" and "Bob")
    /// and one direct chat containing both.
    fn create_mention_test_state() -> (State, Uuid, Identity, Identity) {
        let mut me = Identity::default();
        me.set_username("Alice");

        let mut bob = Identity::default();
        bob.set_username("Bob");

        let me_did = me.did_key();
        let bob_did = bob.did_key();

        let mut identities: HashMap<DID, Identity> = HashMap::new();
        identities.insert(me_did.clone(), me.clone());
        identities.insert(bob_did.clone(), bob.clone());

        let chat_id = Uuid::new_v4();
        let mut participants = HashSet::new();
        participants.insert(me_did);
        participants.insert(bob_did);

        let chat = Chat::new(
            chat_id,
            participants,
            ConversationSettings::Direct(Default::default()),
            None,
            None,
            VecDeque::new(),
            vec![],
        );

        let mut all_chats = HashMap::new();
        all_chats.insert(chat_id, chat);

        let chats = Chats {
            all: all_chats,
            active: Some(chat_id),
            active_media: None,
            in_sidebar: VecDeque::new(),
            favorites: vec![],
            readd_sidebars: false,
        };

        let friends = Friends {
            all: HashSet::new(),
            blocked: HashSet::new(),
            incoming_requests: HashSet::new(),
            outgoing_requests: HashSet::new(),
        };

        let storage = common::state::storage::Storage::default();

        let state = State::mock(me.clone(), identities, chats, friends, storage);
        (state, chat_id, me, bob)
    }

    /// Cas de base de la pipeline de mention : un message contenant
    /// `@<DID>` (où le DID identifie un participant du chat) doit voir
    /// la séquence remplacée par un tag HTML `message-user-tag` portant
    /// le nom d'utilisateur résolu via le State (« @Bob »). Vérifie à
    /// la fois la résolution DID → username et la production du tag.
    /// Premier test où `format_text` reçoit un `State::mock()` non-None,
    /// activant ainsi la branche de traitement des mentions.
    // note: VIDEO
    #[test]
    fn format_text_with_did_mention() {
        let (state, chat_id, _me, bob) = create_mention_test_state();
        let mention = format!("Hello @{} ", bob.did_key());
        let result = format_text(&mention, true, false, Some((&state, &chat_id, false)));

        // The mention should be replaced with a user tag
        assert!(result.contains("message-user-tag"), "Mention should produce a user tag");
        assert!(result.contains("@Bob"), "Tag should contain the username");
        assert!(result.contains(&bob.did_key().to_string()), "Tag should contain the DID");
    }

    /// Vérifie la cohabitation entre le rendu markdown (`**Hi**` →
    /// `<strong>Hi</strong>`) et le remplacement de mention dans le
    /// même message. Les deux transformations doivent s'appliquer
    /// simultanément sans que l'une n'invalide l'autre — important
    /// car les mentions sont parsées à part puis réinjectées dans le
    /// flux markdown.
    // note: VIDEO
    #[test]
    fn format_text_with_did_mention_and_markdown() {
        let (state, chat_id, _me, bob) = create_mention_test_state();
        let mention = format!("**Hi** @{} how are you?", bob.did_key());
        let result = format_text(&mention, true, false, Some((&state, &chat_id, false)));

        // Markdown should be applied
        assert!(result.contains("<strong>"), "Markdown bold should be applied");
        // Mention should be rendered
        assert!(result.contains("message-user-tag"), "Mention should produce a user tag");
        assert!(result.contains("@Bob"), "Tag should contain the username");
    }

    /// Vérifie la combinaison mention + conversion d'émoticône ASCII :
    /// le DID doit être remplacé par un tag utilisateur, ET `:)` doit
    /// être converti en 🙂 dans le même résultat. Confirme que les
    /// deux étapes (résolution mentions, conversion emoji) coexistent
    /// dans la même passe `format_text` sans interférence.
    // note: VIDEO
    #[test]
    fn format_text_with_did_mention_and_emoji() {
        let (state, chat_id, _me, bob) = create_mention_test_state();
        let mention = format!("@{} :)", bob.did_key());
        let result = format_text(&mention, false, true, Some((&state, &chat_id, false)));

        // Mention should be rendered
        assert!(result.contains("message-user-tag"), "Mention should produce a user tag");
        assert!(result.contains("@Bob"), "Tag should contain the username");
        // Emoji should be converted
        assert!(result.contains("🙂"), "Emoji should be converted");
    }

    /// Vérifie la gestion de plusieurs mentions distinctes dans un même
    /// message. Le test construit un State avec trois participants
    /// (Alice, Bob, Charlie) et mentionne explicitement Bob et Charlie.
    /// Les deux DIDs doivent être résolus et donner lieu à deux tags
    /// `message-user-tag` distincts dans la sortie HTML, comptés via
    /// `matches().count()` pour valider l'absence de fusion accidentelle.
    #[test]
    fn format_text_with_multiple_did_mentions() {
        // Create a state with a third participant "Charlie"
        let mut me = Identity::default();
        me.set_username("Alice");

        let mut bob = Identity::default();
        bob.set_username("Bob");

        let mut charlie = Identity::default();
        charlie.set_username("Charlie");

        let me_did = me.did_key();
        let bob_did = bob.did_key();
        let charlie_did = charlie.did_key();

        let mut identities: HashMap<DID, Identity> = HashMap::new();
        identities.insert(me_did.clone(), me.clone());
        identities.insert(bob_did.clone(), bob.clone());
        identities.insert(charlie_did.clone(), charlie.clone());

        let chat_id = Uuid::new_v4();
        let mut participants = HashSet::new();
        participants.insert(me_did);
        participants.insert(bob_did.clone());
        participants.insert(charlie_did.clone());

        let chat = Chat::new(
            chat_id,
            participants,
            ConversationSettings::Direct(Default::default()),
            None,
            None,
            VecDeque::new(),
            vec![],
        );

        let mut all_chats = HashMap::new();
        all_chats.insert(chat_id, chat);

        let chats = Chats {
            all: all_chats,
            active: Some(chat_id),
            active_media: None,
            in_sidebar: VecDeque::new(),
            favorites: vec![],
            readd_sidebars: false,
        };

        let state = State::mock(me.clone(), identities, chats, Friends::default(), common::state::storage::Storage::default());

        let mention = format!("Hey @{} and @{} ", bob.did_key(), charlie.did_key());
        let result = format_text(&mention, true, false, Some((&state, &chat_id, false)));

        // Both mentions should be rendered
        assert!(result.contains("message-user-tag"), "Mentions should produce user tags");
        assert!(result.contains("@Bob"), "First mention should contain Bob");
        assert!(result.contains("@Charlie"), "Second mention should contain Charlie");

        // Count occurrences of message-user-tag — should be 2
        let tag_count = result.matches("message-user-tag").count();
        assert_eq!(tag_count, 2, "Both mentions should be rendered");
    }

    /// Cas négatif fondamental : si le DID mentionné n'appartient pas
    /// aux participants du chat actif, aucun tag de mention ne doit
    /// être produit. La séquence DID brute doit néanmoins être
    /// préservée dans la sortie (le parser ne supprime pas le texte,
    /// il s'abstient simplement de le décorer). Évite que les
    /// utilisateurs puissent faire apparaître les noms d'inconnus
    /// par simple injection de DID arbitraire.
    #[test]
    fn format_text_with_non_participant_did() {
        let (state, chat_id, _me, _bob) = create_mention_test_state();

        // Create a DID that is NOT in the chat participants
        let stranger = Identity::default();
        let mention = format!("Hello @{} ", stranger.did_key());
        let result = format_text(&mention, true, false, Some((&state, &chat_id, false)));

        // The non-participant DID should NOT be turned into a tag
        // (parse_mentions keeps it as-is when no match is found)
        assert!(!result.contains("message-user-tag"), "Non-participant should not be tagged");
        // The DID text should still appear in the output
        assert!(result.contains(&stranger.did_key().to_string()), "DID text should be preserved");
    }

    /// Vérifie qu'une mention de soi-même (l'utilisateur courant
    /// `me`/Alice mentionnant son propre DID) produit également un
    /// tag `message-user-tag` avec le bon username. Pas de
    /// traitement spécial pour le self-mention — tous les
    /// participants sont équivalents du point de vue du résolveur.
    #[test]
    fn format_text_with_self_mention() {
        let (state, chat_id, me, _bob) = create_mention_test_state();
        let mention = format!("Hey @{} ", me.did_key());
        let result = format_text(&mention, true, false, Some((&state, &chat_id, false)));

        // Self-mention should also produce a tag
        assert!(result.contains("message-user-tag"), "Self-mention should produce a user tag");
        assert!(result.contains("@Alice"), "Tag should contain self username");
    }

    /// Vérifie le mode `visual = true` (4ème paramètre du tuple
    /// `Some((state, chat_id, visual))`) : le tag de mention doit
    /// recevoir la classe CSS supplémentaire `visual-only`. Ce mode
    /// est utilisé dans les contextes d'affichage purement visuel
    /// (aperçu sidebar, prévisualisation) où le tag ne doit pas
    /// être interactif (pas de menu contextuel sur clic).
    #[test]
    fn format_text_with_did_mention_visual_mode() {
        let (state, chat_id, _me, bob) = create_mention_test_state();
        let mention = format!("Hi @{} ", bob.did_key());
        // With visual = true
        let result = format_text(&mention, true, false, Some((&state, &chat_id, true)));

        // When visual = true, the tag should have class "visual-only"
        assert!(result.contains("visual-only"), "Visual mode should add visual-only class");
        assert!(result.contains("@Bob"), "Tag should contain the username");
    }

    /// Cas limite de positionnement : la mention apparaît tout au
    /// début de la chaîne, sans caractère préfixe. Vérifie que la
    /// regex de détection de mention ne dépend pas d'un séparateur
    /// gauche (espace, début-de-paragraphe explicite) — le début
    /// de chaîne lui-même doit suffire comme délimiteur.
    #[test]
    fn format_text_with_did_mention_at_start() {
        let (state, chat_id, _me, bob) = create_mention_test_state();
        let mention = format!("@{} look here", bob.did_key());
        let result = format_text(&mention, true, false, Some((&state, &chat_id, false)));

        assert!(result.contains("message-user-tag"), "Mention at start should work");
        assert!(result.contains("@Bob"), "Tag should contain the username");
    }

    /// Symétrique du précédent : la mention apparaît en toute fin
    /// de chaîne, sans caractère suffixe (ni espace ni ponctuation
    /// finale). Vérifie que la regex tolère la fin-de-chaîne comme
    /// délimiteur droit, sans tronquer les derniers caractères du
    /// DID au passage.
    #[test]
    fn format_text_with_did_mention_at_end() {
        let (state, chat_id, _me, bob) = create_mention_test_state();
        let mention = format!("See this @{}", bob.did_key());
        let result = format_text(&mention, true, false, Some((&state, &chat_id, false)));

        assert!(result.contains("message-user-tag"), "Mention at end should work");
        assert!(result.contains("@Bob"), "Tag should contain the username");
    }

    /// Test de comportement contextuel : une mention située à
    /// l'intérieur d'un bloc de code inline (`` `@<DID>` ``) ne
    /// doit PAS être convertie en tag. Choix de design important :
    /// les utilisateurs partageant des exemples de code ou de
    /// configuration mentionnant des DIDs ne doivent pas voir leur
    /// contenu involontairement décoré. La balise `<code>` doit
    /// néanmoins être produite par le rendu markdown.
    #[test]
    fn format_text_with_did_mention_in_code_block() {
        let (state, chat_id, _me, bob) = create_mention_test_state();
        let mention = format!("Code: `@{}`", bob.did_key());
        let result = format_text(&mention, true, false, Some((&state, &chat_id, false)));

        // Mentions inside backtick code blocks should NOT be replaced
        assert!(!result.contains("message-user-tag"), "Mention in code block should not be tagged");
        // The raw DID text should appear inside the code span
        assert!(result.contains("<code>"), "Code block should be rendered");
    }

    /// Vérifie le contrat d'API du paramètre `state` (4ème argument
    /// de `format_text`) : lorsque `None` est passé, aucune
    /// résolution de mention ne doit avoir lieu, indépendamment du
    /// contenu du message. Le DID brut doit être préservé tel quel
    /// dans la sortie. Garantit que le code appelant qui n'a pas
    /// accès à un State (ex : prévisualisations, tests, contextes
    /// de rendu hors-conversation) ne crashe pas et ne produit pas
    /// de tags incohérents.
    #[test]
    fn format_text_without_state_no_mention_replacement() {
        let (_state, _chat_id, _me, bob) = create_mention_test_state();
        let mention = format!("Hello @{} ", bob.did_key());
        // When no state is passed (None), no mention replacement should happen
        let result = format_text(&mention, true, false, None);

        assert!(!result.contains("message-user-tag"), "Without state, no tag should be produced");
        assert!(result.contains(&bob.did_key().to_string()), "Raw DID text should be preserved");
    }

    /// Test de prévention XSS dans le contexte spécifique où un
    /// State est présent (branche de traitement des mentions
    /// activée). Vérifie qu'une tentative d'injection
    /// `<script>alert('xss')</script>` est échappée même lorsque
    /// le pipeline de mentions est actif. Garantit que la branche
    /// avec State ne contourne pas l'échappement HTML appliqué
    /// dans la branche sans State.
    #[test]
    fn format_text_with_mention_and_xss_prevention() {
        let (state, chat_id, _me, _bob) = create_mention_test_state();
        let mention = "<script>alert('xss')</script>";
        let result = format_text(mention, true, false, Some((&state, &chat_id, false)));

        // HTML should be escaped even when state is present
        assert!(result.contains("&lt;script&gt;"), "XSS should be prevented");
        assert!(!result.contains("<script>"), "Raw script tag should not appear");
    }

    /// Test combinant les trois transformations critiques
    /// simultanément : markdown (`**bold**`), tentative XSS
    /// (`<script>x</script>`) et mention DID. Toutes les sorties
    /// attendues doivent coexister : `<strong>` pour le markdown,
    /// `&lt;script&gt;` pour l'échappement HTML, et
    /// `message-user-tag` pour la mention. Test d'intégration
    /// défensif final pour le pipeline complet de mentions.
    #[test]
    fn format_text_with_mention_markdown_and_xss() {
        let (state, chat_id, _me, bob) = create_mention_test_state();
        // Combine mention, markdown, and XSS attempt
        let mention = format!("**bold** <script>x</script> @{}", bob.did_key());
        let result = format_text(&mention, true, false, Some((&state, &chat_id, false)));

        // Markdown works
        assert!(result.contains("<strong>"), "Markdown should be applied");
        // XSS is prevented
        assert!(result.contains("&lt;script&gt;"), "XSS should be prevented");
        // Mention works
        assert!(result.contains("message-user-tag"), "Mention should be rendered");
        assert!(result.contains("@Bob"), "Tag should contain the username");
    }
}
