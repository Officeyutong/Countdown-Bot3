use countdown_bot3::countdown_bot::client::ResultType;
use log::info;

use crate::{exec_impl::ExecuteResult, MathPlugin};
use anyhow::anyhow;
impl MathPlugin {
    pub async fn execute(
        &self,
        code: &str,
        custom_timeout_message: Option<&str>,
    ) -> ResultType<ExecuteResult> {
        let config = self.config.as_ref().unwrap();
        let template = include_str!("template.py");
        info!("Executing {}", code);
        let replaced_template = template.replace("{CODE}", code).replace(
            "{PACKAGES}",
            &serde_json::to_string(&config.latex_packages)?,
        );
        return self
            .handle_exec(&replaced_template, custom_timeout_message)
            .await;
    }
    pub async fn command_solve(&self, args: Vec<String>) -> ResultType<ExecuteResult> {
        let x = args.get(0).ok_or(anyhow!("请提供未知数!"))?;
        let y = args.get(1).ok_or(anyhow!("请提供方程!"))?;
        return self
            .execute(
                &format!("output=solve('{}','{}')", x, y),
                Some("解方程运行超时!"),
            )
            .await;
    }
    pub async fn command_factor(&self, args: Vec<String>) -> ResultType<ExecuteResult> {
        let x = args.get(0).ok_or(anyhow!("请提供要分解的式子"))?;
        return self
            .execute(
                &format!("output=factor('{}')", x),
                Some("分解因式运行超时!"),
            )
            .await;
    }
    pub async fn command_integrate(&self, args: Vec<String>) -> ResultType<ExecuteResult> {
        let x = args.get(0).ok_or(anyhow!("请提供要积分的函数"))?;
        return self
            .execute(&format!("output=integrate('{}')", x), Some("积分运行超时!"))
            .await;
    }
    pub async fn command_diff(&self, args: Vec<String>) -> ResultType<ExecuteResult> {
        let x = args.get(0).ok_or(anyhow!("请提供要求导的函数"))?;

        return self
            .execute(
                &format!("output=differentiate('{}')", x),
                Some("求导运行超时!"),
            )
            .await;
    }
    pub async fn command_series(&self, args: Vec<String>) -> ResultType<ExecuteResult> {
        let x = args.get(0).ok_or(anyhow!("请提供展开点!"))?;
        let y = args.get(1).ok_or(anyhow!("请提供函数!"))?;
        return self
            .execute(
                &format!("output=series('{}','{}')", x, y),
                Some("级数展开运行超时!"),
            )
            .await;
    }
    pub async fn command_plot(&self, args: Vec<String>) -> ResultType<ExecuteResult> {
        let begin = args
            .get(0)
            .ok_or(anyhow!("请提供起始点!"))?
            .parse::<f64>()
            .map_err(|_| anyhow!("请提供合法的起始点!"))?;
        let end = args
            .get(1)
            .ok_or(anyhow!("请提供结束点!"))?
            .parse::<f64>()
            .map_err(|_| anyhow!("请提供合法的结束点!"))?;
        if args.len() < 3 {
            return Err(anyhow!("请提供足够的参数!").into());
        }
        let funcs =
            serde_json::to_string(&args[2..].join(" ").split(",").collect::<Vec<&str>>()).unwrap();
        return self
            .execute(
                &format!("output=plot({},{},{})", begin, end, funcs),
                Some("绘图运行运行超时!"),
            )
            .await;
    }
    pub async fn command_plotpe(&self, args: Vec<String>) -> ResultType<ExecuteResult> {
        let begin = args
            .get(0)
            .ok_or(anyhow!("请提供起始点!"))?
            .parse::<f64>()
            .map_err(|_| anyhow!("请提供合法的起始点!"))?;
        let end = args
            .get(1)
            .ok_or(anyhow!("请提供结束点!"))?
            .parse::<f64>()
            .map_err(|_| anyhow!("请提供合法的结束点!"))?;
        if args.len() < 3 {
            return Err(anyhow!("请提供足够的参数!").into());
        }
        let funcs =
            serde_json::to_string(&args[2..].join(" ").split(",").collect::<Vec<&str>>()).unwrap();
        return self
            .execute(
                &format!("output=plotpe({},{},{})", begin, end, funcs),
                Some("绘图运行运行超时!"),
            )
            .await;
    }
    pub async fn command_latex(&self, args: Vec<String>) -> ResultType<ExecuteResult> {
        let str = args.join(" ");
        let template = format!("import base64\noutput={{'latex':'','python_expr':'','image':render_latex(base64.decodebytes('{}'.encode()).decode())}}",base64::encode(str.as_bytes()));
        return self.execute(&template, Some("LaTeX渲染运行超时!")).await;
    }
}
