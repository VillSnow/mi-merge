use std::collections::HashSet;

use dioxus::prelude::*;

use super::*;
use crate::{common_types::BranchKey, global_state::get_app_model};

#[component]
pub fn Home() -> Element {
    let mut notes = use_signal(|| Vec::<NoteProps>::new());

    spawn(async move {
        let branches = get_app_model().read().await.branches.clone();
        let mut rx = get_app_model()
            .read()
            .await
            .merged_timeline
            .write()
            .await
            .make_column_receiver();

        while let Some(model_notes) = rx.recv().await {
            let mut notes_prop = Vec::new();
            let mut branch_trace = HashSet::<BranchKey>::new();

            for x in model_notes {
                notes_prop.push(NoteProps {
                    original_host: x.original_host,
                    uri: x.uri,
                    avatar_url: x.mi_note.user.avatar_url,
                    user_name: x.mi_note.user.name.unwrap_or(x.mi_note.user.username),
                    note_info: format!(
                        "{} {:?} {:?}",
                        x.mi_note.created_at, x.mi_note.visibility, x.mi_note.local_only
                    ),
                    text: x.mi_note.text.unwrap_or("".to_owned()),
                    reactions: x.reactions.clone(),
                    branch_fragments: branches
                        .iter()
                        .enumerate()
                        .map(|(i, y)| BranchFragment {
                            color: make_color(i),
                            view: if branch_trace.contains(y) {
                                if x.branches.contains(y) {
                                    BranchFragmentView::Full
                                } else {
                                    BranchFragmentView::Skip
                                }
                            } else if x.branches.contains(&y) {
                                BranchFragmentView::Top
                            } else {
                                BranchFragmentView::None
                            },
                        })
                        .collect(),
                });

                branch_trace.extend(x.branches);
            }

            *notes.write() = notes_prop;
        }
    });

    rsx! {
        Column { notes }
    }
}

fn make_color(n: usize) -> String {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;

    let h = 360.0 / (1.0 + phi) * n as f64;
    let h = h % 360.0;
    let s = 1.0;
    let v = 1.0;

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h {
        h if h >= 0.0 && h < 60.0 => (c, x, 0.0),
        h if h >= 60.0 && h < 120.0 => (x, c, 0.0),
        h if h >= 120.0 && h < 180.0 => (0.0, c, x),
        h if h >= 180.0 && h < 240.0 => (0.0, x, c),
        h if h >= 240.0 && h < 300.0 => (x, 0.0, c),
        h if h >= 300.0 && h < 360.0 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    let r = ((r + m) * 256.0).round().min(255.0) as u8;
    let g = ((g + m) * 256.0).round().min(255.0) as u8;
    let b = ((b + m) * 256.0).round().min(255.0) as u8;

    format!("#{r:02x}{g:02x}{b:02x}")
}
