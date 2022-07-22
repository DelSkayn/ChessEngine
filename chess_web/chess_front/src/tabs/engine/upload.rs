use crate::{components::file_upload, Global};
use anyhow::{anyhow, Result};
use futures::FutureExt;
use log::error;
use seed::{attrs, div, h3, input, label, p, prelude::*, textarea, util, C, IF};
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

#[derive(Debug)]
pub enum Msg {
    Upload(file_upload::Msg),
    UploadCompleted(fetch::Result<UploadResponse>),
    Name(String),
    Description(String),
    Submit,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum UploadResponse {
    Ok { id: i32 },
    Err { error: String },
}

pub fn update(msg: Msg, model: &mut Model, global: &mut Global, orders: &mut impl Orders<Msg>) {
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
            if model.name.is_empty() || !model.upload.has_file() || global.user_token.is_none() {
                return;
            }
            let file = model.upload.take_file().unwrap();
            sumbit_form(model, global.user_token.clone().unwrap(), file, orders)
                .map_err(util::error)
                .ok();
            file_upload::update(
                file_upload::Msg::StartUpload,
                &mut model.upload,
                &mut orders.proxy(Msg::Upload),
            );
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

fn sumbit_form(
    model: &Model,
    token: String,
    file: File,
    orders: &mut impl Orders<Msg>,
) -> Result<()> {
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

    orders.perform_cmd(async move { fetch_form(form_data, token).map(Msg::UploadCompleted).await });

    Ok(())
}

async fn fetch_form(form_data: FormData, token: String) -> fetch::Result<UploadResponse> {
    Request::new("/api/v1/engine")
        .body(form_data.into())
        .header(Header::bearer(token))
        .method(Method::Post)
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub fn view(model: &Model, global: &Global) -> Node<Msg> {
    let disabled = model.name.is_empty() || !model.upload.has_file() || global.user_token.is_none();

    div![
        C!["mx-2 my-4 p-2 lg:w-1/2"],
        h3![C!["text-gray-600 font-bold"], "Upload a chess engine"],
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
        div![
            C!["flex items-center"],
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
            IF!(global.user_token.is_none() => p![
                C!["text-red-400 px-2 font-bold"],
                "You need to be logged-in to upload an engine"
            ])
        ],
        model
            .error
            .as_ref()
            .map(|x| { div![C!["bg-red-400 text-gray-100 italic rounded py-1 px-2 "], x] })
    ]
}
