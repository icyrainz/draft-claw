use crate::opt::*;
use imgurs::ImgurClient;

const IMGUR_CLIENT_ID: &str = "IMGUR_CLIENT_ID";

pub async fn upload_image(path: &str) -> Res<String> {
    let client_id = std::env::var(IMGUR_CLIENT_ID).expect("Expect IMGUR client id");

    let client = ImgurClient::new(&client_id);
    let upload_info = client.upload_image(path).await.err_to_str()?;
    if !upload_info.success {
        return Err("Unable to upload image".into());
    }

    Ok(upload_info.data.link)
}
