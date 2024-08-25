use std::{collections::HashSet, ops::Deref};

use chrono::prelude::*;
use dioxus::prelude::*;
use palette::{FromColor, Oklab, Srgb};

use super::*;
use crate::{
    common_types::{BranchKey, DynNoteModel},
    global_state::get_app_model,
};

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
                notes_prop.push(make_note_prop(&x, &branches, &mut branch_trace));

                branch_trace.extend(x.branches);
            }

            *notes.write() = notes_prop;
        }
    });

    rsx! {
        Column { notes }
    }
}

fn make_note_prop(
    x: &DynNoteModel,
    branches: &[BranchKey],
    branch_trace: &mut HashSet<BranchKey>,
) -> NoteProps {
    let renote_header;
    let main_note;
    if let Some(renote) = &x.mi_note.renote {
        renote_header = Some(&x.mi_note);
        main_note = renote.deref();
    } else {
        renote_header = None;
        main_note = &x.mi_note;
    }

    NoteProps {
        original_host: x.original_host.clone(),
        uri: x.uri.clone(),
        avatar_url: main_note.user.avatar_url.clone(),
        user_name: main_note
            .user
            .name
            .clone()
            .unwrap_or(main_note.user.username.clone()),
        note_info: format!(
            "{} {:?} {:?}",
            from_now(&main_note.created_at),
            main_note.visibility,
            main_note.local_only
        ),
        text: main_note.text.clone().unwrap_or("".to_owned()),
        file_thumbnails: main_note
            .files
            .iter()
            .filter_map(|x| x.thumbnail_url.clone())
            .collect(),
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
        renote: renote_header.map(|x| RenoteInfo {
            avatar_url: x.user.avatar_url.clone(),
            user_name: x.user.name.clone().unwrap_or(x.user.username.clone()),
        }),
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

fn from_now(t: &DateTime<chrono::Utc>) -> String {
    let dur = Utc::now() - t;
    let neg = dur < chrono::TimeDelta::zero();
    let dur = dur.abs();
    let s = if dur.subsec_nanos() >= 500 {
        dur.num_seconds() + 1
    } else {
        dur.num_seconds()
    };

    if s < 45 {
        if neg {
            format!("{:}秒後", s)
        } else {
            format!("{}秒前", s)
        }
    } else if s < 45 * 60 {
        let m = (s as f64 / 60.0).round();
        if neg {
            format!("{m}分後")
        } else {
            format!("{m}分前")
        }
    } else if s < 22 * 60 * 60 {
        let h = (s as f64 / (60.0 * 60.0)).round();
        if neg {
            format!("{h}時間後")
        } else {
            format!("{h}時間前")
        }
    } else {
        let d = (s as f64 / (60.0 * 60.0 * 24.0)).round();
        if neg {
            format!("{d}日後")
        } else {
            format!("{d}日前")
        }
    }
}
