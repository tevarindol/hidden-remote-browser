use crate::config::Settings;
use gloo_net::http::Request;
use js_sys::Array;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, FormData, Response};

const DOCUMENT_FILENAME: &str = "send.html";
const PHOTO_FILENAME: &str = "screen.png";

fn js_err(e: JsValue) -> String {
    format!("{e:?}")
}

fn text_blob(content: &str, mime: &str) -> Result<Blob, String> {
    let options = BlobPropertyBag::new();
    options.set_type(mime);
    let parts = Array::new();
    parts.push(&JsValue::from_str(content));
    Blob::new_with_str_sequence_and_options(&JsValue::from(parts), &options).map_err(js_err)
}

fn file_form(chat_id: &str, name: &str, blob: &Blob, filename: &str) -> Result<FormData, String> {
    let form = FormData::new().map_err(js_err)?;
    form.append_with_str("chat_id", chat_id)
        .map_err(js_err)?;
    form.append_with_blob_and_filename(name, blob, filename)
        .map_err(js_err)?;
    Ok(form)
}

async fn post(url: &str, form: FormData) -> Result<(), String> {
    let response = Request::post(url)
        .body(form)
        .map_err(|error| error.to_string())?
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.ok() {
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| error.to_string())?;
        return Err(format!("telegram api returned {status}: {text}"));
    }
    Ok(())
}

async fn blob_from_data_url(data_url: &str) -> Result<Blob, String> {
    let response = Request::get(data_url)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let response: Response = response.into();
    let blob_value = JsFuture::from(response.blob().map_err(js_err)?)
        .await
        .map_err(js_err)?;
    blob_value.dyn_into::<Blob>().map_err(js_err)
}

pub async fn send(
    settings: &Settings,
    html: &str,
    screenshot_data_url: Option<&str>,
) -> Result<(), String> {
    let base = format!("https://api.telegram.org/bot{}", settings.telegram_token);

    let html_blob = text_blob(html, "text/html")?;
    let form = file_form(&settings.chat_id, "document", &html_blob, DOCUMENT_FILENAME)?;
    post(&format!("{base}/sendDocument"), form).await?;

    if let Some(data_url) = screenshot_data_url {
        let blob = blob_from_data_url(data_url).await?;
        let form = file_form(&settings.chat_id, "photo", &blob, PHOTO_FILENAME)?;
        post(&format!("{base}/sendPhoto"), form).await?;
    }

    Ok(())
}
