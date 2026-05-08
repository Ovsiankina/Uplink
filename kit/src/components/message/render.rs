use std::path::PathBuf;
use std::{collections::HashSet, str::FromStr};

use arboard::Clipboard;
use common::language::{get_local_text, get_local_text_with_args};
use common::state::pending_message::{FileLocation, FileProgression};
use common::state::{Action, Identity, State, ToastNotification};
use common::warp_runner::{thumbnail_to_base64, MultiPassCmd, WarpCmd};
use common::{state::pending_message::progress_file, WARP_CMD_CH};
use derive_more::Display;
use dioxus::prelude::*;
use futures::StreamExt;
use uuid::Uuid;
use warp::error::Error;
use warp::{constellation::file::File, crypto::DID};

use tracing::log;

use common::icons::outline::Shape as Icon;

use crate::components::context_menu::{ContextItem, ContextMenu, IdentityHeader};
use crate::components::embeds::link_embed::EmbedLinks;
use crate::elements::button::Button;
use crate::{components::embeds::file_embed::FileEmbed, elements::textarea};

use super::{format_text, wrap_links_with_a_tags, Order, ReactionAdapter};

#[derive(Props)]
pub struct Props<'a> {
    // Message ID
    id: String,
    // indicates that the message is being edited
    editing: bool,

    // An optional field that, if set to true, will add a CSS class of "loading" to the div element.
    loading: Option<bool>,

    // An optional field that, if set, will be used as the content of a nested div element with a class of "content".
    with_content: Option<Element<'a>>,

    // An optional field that, if set, will be used as the text content of a nested p element with a class of "text".
    with_text: Option<String>,

    reactions: Vec<ReactionAdapter>,

    // An optional field that, if set to true, will add a CSS class of "remote" to the div element.
    remote: Option<bool>,

    // An optional field that, if set, will be used to determine the ordering of the div element relative to other Message elements.
    // The value will be converted to a string using the Order enum's fmt::Display implementation and used as a CSS class of the div element.
    // If not set, the default value of Order::Last will be used.
    order: Option<Order>,

    // available for download
    attachments: Option<Vec<File>>,

    // attachments which are being downloaded
    #[props(!optional)]
    attachments_pending_download: Option<HashSet<File>>,

    /// called when an attachment is downloaded
    on_download: EventHandler<'a, (File, Option<PathBuf>)>,

    /// called when editing is completed
    on_edit: EventHandler<'a, String>,

    /// If true, the markdown parser will be rendered
    parse_markdown: bool,
    transform_ascii_emojis: bool,
    // called when a reaction is clicked
    on_click_reaction: EventHandler<'a, String>,

    // Indicates whether this message is pending to be uploaded or not
    pending: bool,

    // Progress for attachments which are being uploaded
    #[props(!optional)]
    attachments_pending_uploads: Option<&'a Vec<(FileLocation, FileProgression)>>,
    on_resend: Option<EventHandler<'a, (Option<String>, FileLocation)>>,
    on_delete: Option<EventHandler<'a, FileLocation>>,

    pinned: bool,

    is_mention: bool,

    state: &'a UseSharedState<State>,

    chat: Uuid,
}

#[allow(non_snake_case)]
pub fn Message<'a>(cx: Scope<'a, Props<'a>>) -> Element<'a> {
    let loading = cx.props.loading.unwrap_or_default();
    let is_remote = cx.props.remote.unwrap_or_default();
    let order = cx.props.order.unwrap_or(Order::Last);

    let remote_class = "";
    let reactions_class = format!("message-reactions-container {remote_class}");

    let has_attachments = cx
        .props
        .attachments
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    let attachment_list = cx.props.attachments.as_ref().map(|vec| {
        vec.iter().map(|file| {
            let key = file.id();
            rsx!(FileEmbed {
                key: "{key}",
                filename: file.name(),
                filesize: file.size(),
                thumbnail: thumbnail_to_base64(file),
                big: true,
                remote: is_remote,
                with_download_button: true,
                download_pending: cx
                    .props
                    .attachments_pending_download
                    .as_ref()
                    .map(|x| x.contains(file))
                    .unwrap_or(false),
                on_press: move |temp_dir_option| cx
                    .props
                    .on_download
                    .call((file.clone(), temp_dir_option)),
            })
        })
    });

    let single = cx
        .props
        .attachments_pending_uploads
        .map(|v| v.len() < 2)
        .unwrap_or_default();

    let pending_attachment_list = cx.props.attachments_pending_uploads.as_ref().map(|vec| {
        vec.iter().map(|(location, prog)| {
            let file = progress_file(prog);
            rsx!(FileEmbed {
                key: "{file}",
                filename: file,
                remote: is_remote,
                download_pending: false,
                with_download_button: false,
                progress: prog,
                on_press: move |_| {},
                on_resend_msg: move |_| {
                    if single {
                        if let Some(e) = &cx.props.on_resend {
                            e.call((cx.props.with_text.clone(), location.clone()))
                        }
                    } else {
                        if let Some(e) = &cx.props.on_delete {
                            e.call(location.clone())
                        }
                        if let Some(e) = &cx.props.on_resend {
                            e.call((None, location.clone()))
                        }
                    }
                },
                on_delete_msg: move |_| {
                    if let Some(e) = &cx.props.on_delete {
                        e.call(location.clone())
                    }
                },
            })
        })
    });

    let loading_class = loading.then_some("loading").unwrap_or_default();
    let remote_class = is_remote.then_some("remote").unwrap_or_default();
    let mention_class = cx.props.is_mention.then_some("mention").unwrap_or_default();
    let order_class = order.to_string();
    let msg_pending_class = cx
        .props
        .pending
        .then_some("message-pending")
        .unwrap_or_default();
    let is_editing = cx.props.with_text.is_some() && cx.props.editing;

    cx.render(rsx! (
        cx.props.pinned.then(|| {
            rsx!(div {
                class: "pin-indicator",
                aria_label: "pin-indicator",
                common::icons::Icon {
                    ..common::icons::IconProps {
                        class: None,
                        size: 14,
                        fill:"currentColor",
                        icon: Icon::Pin,
                        disabled: false,
                        disabled_fill: "#9CA3AF"
                    },
                },
            })
        }),
        is_editing.then(||
            rsx! (
                div {
                    class: "edit-message-wrap",
                    onclick: move |_| {
                        cx.props.on_edit.call(cx.props.with_text.clone().unwrap_or_default());
                    }
                },
            )
        ),
        div {
            class: {
                format_args!(
                    "message {} {} {} {} {} {}",
                   loading_class, remote_class, order_class, msg_pending_class, mention_class, if is_editing { "edit-message" } else { "" }
                )
            },
            aria_label: {
                format_args!(
                    "message-{}",
                    if is_remote {
                        "remote"
                    } else { "local" },
                )
            },
            white_space: "pre-wrap",
            (cx.props.with_content.is_some()).then(|| rsx! (
                    div {
                    class: "content",
                    cx.props.with_content.as_ref(),
                },
            )),
            is_editing.then(||
                rsx! (
                    p {
                        class: "text",
                        aria_label: "message-text",
                        rsx! (
                            EditMsg {
                                id: cx.props.id.clone(),
                                text: cx.props.with_text.clone().unwrap_or_default(),
                                on_enter: move |update| {
                                    cx.props.on_edit.call(update);
                                }
                            }
                        )
                    }
                )
            ),
            (cx.props.with_text.is_some() && !cx.props.editing).then(|| rsx!(
                ChatText {
                    text: cx.props.with_text.as_ref().cloned().unwrap_or_default(),
                    remote: is_remote,
                    pending: cx.props.pending,
                    markdown: cx.props.parse_markdown,
                    state: cx.props.state,
                    chat: cx.props.chat,
                    ascii_emoji: cx.props.transform_ascii_emojis,
                }
            )),
            has_attachments.then(|| {
                rsx!(
                    div {
                        class: "attachment-list",
                        attachment_list.map(|list| {
                            rsx!( list )
                        })
                    }
                )
            })
            pending_attachment_list.map(|node| {
                rsx!(node)
            })
        },
        div {
            class: "{reactions_class}",
            aria_label: "message-reactions-container",
            cx.props.reactions.iter().map(|reaction| {
                let reaction_count = reaction.reaction_count;
                let emoji = &reaction.emoji;
                let alt = &reaction.alt;

                rsx!(
                    div {
                         alt: "{alt}",
                        class:
                            format_args!("emoji-reaction {}", if reaction.self_reacted {
                            "emoji-reaction-self"
                        } else { "" }),
                        aria_label: {
                            format_args!(
                                "emoji-reaction-{}",
                                if reaction.self_reacted {
                                    "self"
                                } else { "remote" }
                            )
                        },
                        onclick: move |_| {
                            cx.props.on_click_reaction.call(emoji.clone());
                        },
                        "{emoji} {reaction_count}"
                    }
                )
            })
        }
    ))
}

#[derive(Props)]
struct EditProps<'a> {
    id: String,
    text: String,
    on_enter: EventHandler<'a, String>,
}

#[allow(non_snake_case)]
fn EditMsg<'a>(cx: Scope<'a, EditProps<'a>>) -> Element<'a> {
    log::trace!("rendering EditMsg");

    cx.render(rsx!(textarea::InputRich {
        id: cx.props.id.clone(),
        aria_label: "edit-message-input".into(),
        ignore_focus: false,
        value: cx.props.text.clone(),
        onchange: move |_| {},
        onreturn: move |(s, is_valid, _): (String, bool, _)| {
            if is_valid && !s.is_empty() {
                cx.props.on_enter.call(s);
            } else {
                cx.props.on_enter.call(cx.props.text.clone());
            }
        }
    }))
}

#[derive(Props)]
pub struct ChatMessageProps<'a> {
    text: String,
    remote: bool,
    pending: bool,
    markdown: bool,
    ascii_emoji: bool,
    state: &'a UseSharedState<State>,
    chat: Uuid,
}

#[allow(non_snake_case)]
pub fn ChatText<'a>(cx: Scope<'a, ChatMessageProps<'a>>) -> Element<'a> {
    // DID::from_str panics if text is 'z'. simple fix is to ensure string is long enough.
    if cx.props.text.len() > 2 {
        if let Ok(id) = DID::from_str(&cx.props.text) {
            return cx.render(rsx!(IdentityMessage { id: id }));
        }
    }

    let formatted_text = format_text(
        &cx.props.text,
        cx.props.markdown,
        cx.props.ascii_emoji,
        Some((&cx.props.state.read(), &cx.props.chat, false)),
    );
    let (formatted_text, links) = wrap_links_with_a_tags(&formatted_text);

    let text_type_class = if cx.props.pending {
        "pending-text"
    } else {
        "text"
    };

    cx.render(rsx!(
        div {
            class: text_type_class,
            p {
                class: text_type_class,
                aria_label: "message-text-{cx.props.text}",
                dangerous_inner_html: "{formatted_text}",
            },
            links.first().and_then(|l| cx.render(rsx!(
                EmbedLinks {
                    link: l.to_string(),
                    remote: cx.props.remote
                })
            ))
        }
    ))
}

#[derive(Display)]
pub enum IdentityCmd {
    #[display(fmt = "GetIdentity")]
    GetIdentity(DID),
    #[display(fmt = "SentFriendRequest")]
    SentFriendRequest(String, Vec<Identity>),
}

#[derive(Props, PartialEq)]
pub struct IdentityMessageProps {
    id: DID,
}

#[allow(non_snake_case)]
pub fn IdentityMessage(cx: Scope<IdentityMessageProps>) -> Element {
    let state = use_shared_state::<State>(cx)?;
    let identity = use_state(cx, || None);
    let ch = use_coroutine(cx, |mut rx: UnboundedReceiver<IdentityCmd>| {
        to_owned![identity, state];
        async move {
            let warp_cmd_tx = WARP_CMD_CH.tx.clone();
            while let Some(cmd) = rx.next().await {
                match cmd {
                    IdentityCmd::GetIdentity(id) => {
                        let (tx, rx) = futures::channel::oneshot::channel();
                        let _ = warp_cmd_tx.send(WarpCmd::MultiPass(MultiPassCmd::GetIdentity {
                            did: id,
                            rsp: tx,
                        }));
                        let r = rx.await.expect("no identity found");
                        if let Ok(id) = r {
                            identity.set(Some(id));
                        }
                    }
                    IdentityCmd::SentFriendRequest(id, outgoing_requests) => {
                        let (tx, rx) = futures::channel::oneshot::channel();
                        let _ = warp_cmd_tx.send(WarpCmd::MultiPass(MultiPassCmd::RequestFriend {
                            id,
                            outgoing_requests,
                            rsp: tx,
                        }));
                        let res = rx.await.expect("failed to get response from warp_runner");
                        match res {
                            Ok(_) => {}
                            Err(e) => match e {
                                Error::PublicKeyIsBlocked => {
                                    log::warn!("add friend failed: {}", e);
                                    state.write().mutate(Action::AddToastNotification(
                                        ToastNotification::init(
                                            "".into(),
                                            get_local_text("friends.key-blocked"),
                                            None,
                                            2,
                                        ),
                                    ));
                                }
                                _ => {
                                    log::error!("add friend failed: {}", e);
                                    state.write().mutate(Action::AddToastNotification(
                                        ToastNotification::init(
                                            "".into(),
                                            get_local_text("friends.add-failed"),
                                            None,
                                            2,
                                        ),
                                    ));
                                }
                            },
                        }
                    }
                }
            }
        }
    });
    use_effect(cx, &cx.props.id, |id| {
        to_owned![ch];
        async move {
            ch.send(IdentityCmd::GetIdentity(id));
        }
    });
    match identity.as_ref() {
        Some(identity) => {
            let disabled = state
                .read()
                .outgoing_fr_identities()
                .iter()
                .any(|req| req.did_key().eq(&identity.did_key()))
                || state
                    .read()
                    .get_own_identity()
                    .did_key()
                    .eq(&identity.did_key())
                || state
                    .read()
                    .friend_identities()
                    .iter()
                    .any(|req| req.did_key().eq(&identity.did_key()));

            let short_id = identity.short_id();
            let did_key = identity.did_key();
            let username = identity.username();
            let short_name = format!("{}#{}", username, short_id);
            let random_uuid = Uuid::new_v4().to_string();

            return cx.render(rsx!(
                ContextMenu {
                    key: "{short_id}-{random_uuid}",
                    id: format!("{short_id}-{random_uuid}"),
                    devmode: state.read().configuration.developer.developer_mode,
                    items: cx.render(rsx!(
                        ContextItem {
                            icon: Icon::UserCircle,
                            aria_label: "copy-user-id-from-user-identity-on-chat".into(),
                            text: get_local_text("settings-profile.copy-id"),
                            onpress: move |_| {
                                match Clipboard::new() {
                                    Ok(mut c) => {
                                        if let Err(e) = c.set_text(short_name.clone()) {
                                            log::warn!("Unable to set text to clipboard: {e}");
                                        }
                                    },
                                    Err(e) => {
                                        log::warn!("Unable to create clipboard reference: {e}");
                                    }
                                };
                                state
                                    .write()
                                    .mutate(Action::AddToastNotification(ToastNotification::init(
                                        "".into(),
                                        get_local_text("friends.copied-did"),
                                        None,
                                        2,
                                    )));
                            }
                        },
                        ContextItem {
                            icon: Icon::Key,
                            aria_label: "copy-user-did-key-from-user-identity-on-chat".into(),
                            disabled: false,
                            text: get_local_text("settings-profile.copy-did"),
                            onpress: move |_| {
                                match Clipboard::new() {
                                    Ok(mut c) => {
                                        if let Err(e) = c.set_text(did_key.to_string()) {
                                            log::warn!("Unable to set text to clipboard: {e}");
                                        }
                                    },
                                    Err(e) => {
                                        log::warn!("Unable to create clipboard reference: {e}");
                                    }
                                };
                                state
                                    .write()
                                    .mutate(Action::AddToastNotification(ToastNotification::init(
                                        "".into(),
                                        get_local_text("friends.copied-did"),
                                        None,
                                        2,
                                    )));
                            },
                            tooltip: None,
                        }
                    )),
                   children: cx.render(rsx!(div { // TODO: This needs to be moved to kit/src/components/embeds/identity_embed/mod.rs.
                        class: "embed-identity",
                        IdentityHeader {
                            sender_did: identity.did_key(),
                            with_status: false,
                        },
                        div {
                            class: "profile-container",
                            div {
                                id: "profile-name",
                                aria_label: "profile-name",
                                p {
                                    class: "text",
                                    aria_label: "profile-name-value",
                                    format!("{}", identity.username())
                                }
                            }
                            identity.status_message().and_then(|s|{
                                cx.render(rsx!(
                                    div {
                                        id: "profile-status",
                                        aria_label: "profile-status",
                                        p {
                                            class: "text",
                                            aria_label: "profile-status-value",
                                            s
                                        }
                                    }
                                ))
                            }),
                        },
                        Button {
                            aria_label: String::from("embed-identity-button"),
                            disabled: disabled,
                            with_title: false,
                            onpress: move |_| {
                                ch.send(IdentityCmd::SentFriendRequest(identity.did_key().to_string(), state.read().outgoing_fr_identities()));
                            },
                            icon: if disabled {
                                Icon::Check
                            } else {
                                Icon::Plus
                            },
                            text: if disabled {
                                get_local_text("friends.already-friends")
                            } else {
                                get_local_text_with_args("friends.add-name", vec![("name", identity.username())])
                            },
                            appearance: crate::elements::Appearance::Primary
                        }
                    }))
                }
            ));
        }
        None => {
            return cx.render(rsx!(div {
                class: "embed-identity",
                div {
                    class: "profile-container empty-profile",
                    div {
                        class: "unknown-user",
                        aria_label: "unknown-user",
                        p {
                            class: "text",
                            aria_label: "unknown-user-value",
                            get_local_text("messages.unknown-identity")
                        }
                    },
                    div {
                        id: "unknown-user-did",
                        aria_label: "unknown-user-did",
                        p {
                            class: "text",
                            aria_label: "unknown-user-did-value",
                            cx.props.id.to_string()
                        }
                    }
                }
            }))
        }
    }
}
