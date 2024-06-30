use std::collections::HashSet;

use dioxus::prelude::*;
use palette::{FromColor, Oklab, Srgb};

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
    let l = 0.5;
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let h = std::f64::consts::TAU / (1.0 + phi) * n as f64;
    let h = h % std::f64::consts::TAU;
    let r = 0.25;
    let lab = Oklab::new(l, r * h.cos(), r * h.sin());
    let rgb = Srgb::from_color(lab);
    let r = (rgb.red * 256.0).round().min(255.0) as u8;
    let g = (rgb.green * 256.0).round().min(255.0) as u8;
    let b = (rgb.blue * 256.0).round().min(255.0) as u8;
    return format!("#{r:02x}{g:02x}{b:02x}");
}
