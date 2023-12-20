use crate::file_upload;
use anyhow::{anyhow, Result};
use futures::FutureExt;
use log::error;
use seed::{attrs, div, h3, input, label, prelude::*, textarea, util, C, IF};
use serde::Deserialize;
use web_sys::{File, FormData};

#[derive(Debug)]
pub struct Model {
    name: String,
    description: String,
    upload: file_upload::Model,
    error: Option<String>,
}

impl Model {
    pub fn new() -> Self {
        Model {
            name: String::new(),
            description: String::new(),
            upload: file_upload::Model::new("Select engine file".to_string()),
            error: None,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum UploadResponse {
    Ok { ok: bool },
    Err { error: String },
}

#[derive(Debug)]
pub enum Msg {
    Upload(file_upload::Msg),
    UploadCompleted(fetch::Result<UploadResponse>),
    Name(String),
    Description(String),
    Submit,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Name(x) => {
            model.name = x;
        }
        Msg::Description(x) => {
            model.description = x;
        }
        Msg::Upload(msg) => {
            file_upload::update(msg, &mut model.upload, &mut orders.proxy(Msg::Upload));
        }
        Msg::Submit => {
            model.error = None;
            if model.name.is_empty() || !model.upload.has_file() {
                return;
            }
            let file = model.upload.take_file().unwrap();
            sumbit_form(model, file, orders).map_err(util::error).ok();
            orders.skip();
        }
        Msg::UploadCompleted(resp) => match resp {
            Ok(UploadResponse::Ok { .. }) => {}
            Ok(UploadResponse::Err { error }) => model.error = Some(error),
            Err(e) => {
                model.error = Some(
                    "Error uploading to the server, something wrong with the connection?"
                        .to_string(),
                );
                error!("{:?}", e);
            }
        },
    }
}

fn sumbit_form(model: &Model, file: File, orders: &mut impl Orders<Msg>) -> Result<()> {
    let form_data = FormData::new().map_err(|_| anyhow!("Failed to create form data"))?;
    form_data
        .append_with_str("name", &model.name)
        .map_err(|_| anyhow!("Could not append name to form data"))?;
    form_data
        .append_with_str("description", &model.description)
        .map_err(|_| anyhow!("Could not append description to form data"))?;
    form_data
        .append_with_blob_and_filename("file", &*file, file.name().as_str())
        .map_err(|_| anyhow!("Could not append file to the form data"))?;

    orders.perform_cmd(async move { fetch_form(form_data).map(Msg::UploadCompleted).await });

    Ok(())
}

async fn fetch_form(form_data: FormData) -> fetch::Result<UploadResponse> {
    Request::new("/api/v1/engine")
        .body(form_data.into())
        .method(Method::Post)
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub fn view(model: &Model) -> Node<Msg> {
    let disabled = model.name.is_empty() || !model.upload.has_file();

    div![
        h3![C!["text-gray-600 font-bold"], "Upload a engine"],
        label![
            C!["flex flex-col italic text-gray-600 items-start"],
            "Name",
            input![
                C!["p-1 px-2 text-md shadow-inner rounded my-1 border border-gray-400"],
                attrs! {
                    At::Type => "text",
                    At::Placeholder => "Engine Name",
                    At::Value => model.name,
                },
                input_ev(Ev::Input, Msg::Name),
            ],
        ],
        label![
            C!["flex flex-col italic text-gray-600"],
            "Description",
            textarea![
                C!["w-full p-1 text-sm shadow-inner rounded my-1 border border-gray-400"],
                attrs! {
                    At::Type => "textarea",
                    At::Placeholder => "Description",
                    At::Value => model.description,
                },
                input_ev(Ev::Input, Msg::Description),
            ],
        ],
        file_upload::view(&model.upload).map_msg(Msg::Upload),
        input![
            attrs! {
                At::Type => "button",
                At::Value => "Upload",
                At::Disabled => disabled.as_at_value()
            },
            C!["text-sm transition-all p-1
                shadow rounded my-1 px-2 py-1 bg-green-400 
                border border-green-400 text-gray-100 
                hover:bg-green-500 disabled:bg-gray-400 
                disabled:border-gray-400 disabled:text-gray-200"],
            IF!(!disabled => ev(Ev::Click, |_| Msg::Submit)),
        ],
        model
            .error
            .as_ref()
            .map(|x| { div![C!["bg-red-400 text-gray-100 italic rounded py-1 px-2 "], x] })
    ]
}
