use seed::{attrs, path, prelude::*, svg, C};

pub enum Icon {
    Pencil,
    Refresh,
    Trash,
}

pub fn icon<M>(icon: Icon, class: &str) -> Node<M> {
    let content = match icon {
        Icon::Pencil => path![attrs! {
            At::StrokeLinecap => "round",
            At::StrokeLineJoin=> "round",
            At::D => "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z",
        }],
        Icon::Refresh => path![attrs! {
            At::StrokeLinecap => "round",
            At::StrokeLineJoin=> "round",
            At::D => "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15",
        }],
        Icon::Trash => path![attrs! {
            At::StrokeLinecap => "round",
            At::StrokeLineJoin=> "round",
            At::D => "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
        }],
    };
    svg![
        C![class],
        attrs! {
            At::Fill => "none",
            At::ViewBox => "0 0 24 24",
            At::Stroke => "currentColor",
            At::StrokeWidth => "2",
        },
        content
    ]
}
