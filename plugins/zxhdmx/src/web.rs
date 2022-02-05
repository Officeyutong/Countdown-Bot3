use std::path::PathBuf;

use crate::{config::ZxhdmxConfig, DataType, HTMLTemplateType};
use countdown_bot3::countdown_bot::client::ResultType;
use salvo::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
pub struct GetDataHandler {
    pub(crate) data: DataType,
    pub(crate) config: ZxhdmxConfig,
}

#[async_trait::async_trait]
impl Handler for GetDataHandler {
    async fn handle(
        &self,
        req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        _ctrl: &mut FlowCtrl,
    ) {
        if let Err(e) = self.handle_req(req, res).await {
            res.render_json(&json!({
                "code": -1,
                "message": format!("{}",e).as_str()
            }))
        }
    }
}
impl GetDataHandler {
    async fn handle_req(&self, req: &mut Request, res: &mut Response) -> ResultType<()> {
        // debug!("Received form: {:#?}", req);
        #[derive(Deserialize)]
        struct Req {
            pub password: String,
        }
        let payload: Req = req.read_from_json().await?;

        if !self.config.verify_password(&payload.password) {
            res.render_json(&json! ({
                "code": -1i32 ,
                "message": "密码错误!"
            }));
            return Ok(());
        }
        let data = self.data.read().await;
        res.render_json(&json! ({
            "code": 0i32 ,
            "data": *data
        }));
        return Ok(());
    }
}

pub struct SetDataHandler {
    pub(crate) data: DataType,
    pub(crate) config: ZxhdmxConfig,
    pub(crate) data_dir: PathBuf,
}

#[async_trait::async_trait]
impl Handler for SetDataHandler {
    async fn handle(
        &self,
        req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        _ctrl: &mut FlowCtrl,
    ) {
        if let Err(e) = self.handle_req(req, res).await {
            res.render_json(&json!({
                "code": -1,
                "message": format!("{}",e).as_str()
            }))
        }
    }
}
impl SetDataHandler {
    async fn handle_req(&self, req: &mut Request, res: &mut Response) -> ResultType<()> {
        #[derive(Deserialize)]
        struct Req {
            pub password: String,
            pub data: String,
        }
        let payload: Req = req.read_from_json().await?;
        if !self.config.verify_password(&payload.password) {
            res.render_json(&json! ({
                "code": -1i32 ,
                "message": "密码错误!"
            }));
            return Ok(());
        }
        let data: Value = serde_json::from_str::<Value>(&payload.data)?;
        let mut game_data = self.data.write().await;
        *game_data = data;
        let str_to_write = serde_json::to_string(&*game_data)?;
        tokio::fs::write(self.data_dir.join("data.json"), str_to_write).await?;
        res.render_json(&json!({
            "code": 0,
            "message": "操作完成!"
        }));
        return Ok(());
    }
}

pub struct TemplateGetter {
    pub(crate) data: HTMLTemplateType,
}
#[async_trait::async_trait]
impl Handler for TemplateGetter {
    async fn handle(
        &self,
        _req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        _ctrl: &mut FlowCtrl,
    ) {
        res.render_html_text(self.data.lock().unwrap().as_str());
    }
}
