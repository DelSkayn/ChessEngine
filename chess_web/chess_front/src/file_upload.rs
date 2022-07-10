use seed::{attrs, div, input, label, prelude::*, util, C};
use web_sys::{File, HtmlInputElement};

#[derive(Debug)]
pub struct Model {
    file: Option<File>,
    file_name: Option<String>,
    placeholder: String,
}

impl Model {
    pub fn new(placeholder: String) -> Self {
        Model {
            file: None,
            file_name: None,
            placeholder,
        }
    }

    pub fn take_file(&mut self) -> Option<File> {
        self.file.take()
    }

    pub fn has_file(&self) -> bool {
        self.file.is_some()
    }
}

#[derive(Debug)]
pub enum Msg {
    FileChanged(Option<File>),
    Upload,
    StartUpload,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FileChanged(x) => {
            model.file_name = x.as_ref().map(|x| x.name());
            model.file = x;
        }
        Msg::Upload => {
            orders.skip();
        }
        Msg::StartUpload => {
            model.file = None;
            model.file_name = None;
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    fn extract_file(target: &web_sys::EventTarget) -> Result<Option<File>, &'static str> {
        target
            .dyn_ref::<HtmlInputElement>()
            .ok_or("Element not an input element")
            .and_then(|x| {
                x.files()
                    .ok_or("Failed to extract files from input elements")
            })
            .map(|x| x.item(0))
    }

    fn handle_change(e: web_sys::Event) -> Option<Msg> {
        e.target()
            .as_ref()
            .ok_or("Can't get event target reference")
            .and_then(extract_file)
            .map(Msg::FileChanged)
            .map_err(util::error)
            .ok()
    }

    label![
        C!["text-sm rounded overflow-hidden border border-green-400 text-gray-500 flex items-center group bg-white"],
        input![
            C!["hidden"],
            attrs! {
                At::Type => "file",
            },
            ev(Ev::Change, handle_change)
        ],
        div![
            C!["bg-green-400 text-gray-100 text-md px-2 py-1 shadow mr-2 group-hover:bg-green-500"],
            "Browse"
        ],
        model
            .file_name
            .as_ref()
            .map(|x| x.as_str())
            .unwrap_or(&model.placeholder)
    ]
}
