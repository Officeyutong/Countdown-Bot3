use countdown_bot3::countdown_bot::client::CountdownBotClient;
use log::{debug, info};
use pyvm::{
    builtins::{PyDict, PyDictRef, PyInt, PyIntRef, PyStr, PyStrRef},
    function::{IntoPyObject, KwArgs},
    pyclass, pyimpl, IntoPyRef, PyRef, PyResult, PyValue, VirtualMachine,
};
use rustpython_vm as pyvm;
use std::{
    fmt::Debug,
};

use crate::utils::bigint_to_i64;

#[pyclass(module = false, name = "MyPyLogger")]
#[derive(PyValue, Debug)]
pub struct MyPyLogger;

#[pyimpl]
impl MyPyLogger {
    #[pymethod]
    pub fn info(&self, log: PyStrRef) {
        info!("{}", log.as_str());
    }
    #[pymethod]
    pub fn debug(&self, log: PyStrRef) {
        info!("{}", log.as_str());
    }
}

#[pyclass(module = false, name = "WrappedCountdownBot")]
#[derive(PyValue)]
pub struct WrappedCountdownBot {
    pub logger: PyRef<MyPyLogger>,
    client: CountdownBotClient,
}
impl Debug for WrappedCountdownBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrappedCountdownBot")
            .field("logger", &self.logger)
            .field("client", &"CountdownBotClient".to_string())
            .finish()
    }
}
#[pyimpl]
impl WrappedCountdownBot {
    pub fn new(logger: PyRef<MyPyLogger>, client: CountdownBotClient) -> Self {
        Self { logger, client }
    }
    #[pyproperty]
    pub fn logger(&self) -> PyRef<MyPyLogger> {
        return self.logger.clone();
    }
    #[pymethod]
    pub fn send_group_msg(&self, mut kwargs: KwArgs, vm: &VirtualMachine) -> PyResult<PyIntRef> {
        let message = kwargs
            .pop_kwarg("message")
            .ok_or(vm.new_value_error("'message' expected".to_string()))?
            .downcast::<PyStr>()
            .map_err(|_| vm.new_value_error("str expected".to_string()))?;
        let gid = bigint_to_i64(
            kwargs
                .pop_kwarg("group_id")
                .ok_or(vm.new_value_error("'group' expected".to_string()))?
                .downcast::<PyInt>()
                .map_err(|_| vm.new_value_error("int expected".to_string()))?
                .as_bigint(),
        );

        debug!("Send message to {}:\n{}", gid, message);
        
        // let client = self.client.clone();
        // let msg0 = message.clone();
        // let mut fut = Box::pin(TokioContext::new(
        //     client.send_group_msg(gid as i64, msg0.as_str(), false),
        //     handle,
        // ));
        // let mid = tokio::runtime::Runtime::new()
        // .unwrap()
        // .block_on()
        let mid = 
        // (loop {
        //     match fut.as_mut().poll(&mut Context::from_waker(&noop_waker())) {
        //         Poll::Ready(v) => break v,
        //         Poll::Pending => {
        //             debug!("Pending..");
        //             continue;
        //         }
        //     }
        // })
        self.client.send_group_msg_sync(gid, message.as_str(), false)
        .map_err(|e| vm.new_value_error(format!("Failed to send message: {}", e)))?
        .message_id;
        return Ok(PyInt::from(mid).into_ref(vm));
    }
    #[pymethod]
    pub fn get_group_member_info(
        &self,
        mut kwargs: KwArgs,
        vm: &VirtualMachine,
    ) -> PyResult<PyDictRef> {
        let user_id = bigint_to_i64(
            kwargs
                .pop_kwarg("user_id")
                .ok_or(vm.new_value_error("'user_id' expected".to_string()))?
                .downcast::<PyInt>()
                .map_err(|_| vm.new_value_error("int expected".to_string()))?
                .as_bigint(),
        );
        let group_id = bigint_to_i64(
            kwargs
                .pop_kwarg("group_id")
                .ok_or(vm.new_value_error("'group_id' expected".to_string()))?
                .downcast::<PyInt>()
                .map_err(|_| vm.new_value_error("int expected".to_string()))?
                .as_bigint(),
        );
        let resp = self.client.get_group_member_info_sync(group_id, user_id, false)
        .map_err(|e| vm.new_value_error(format!("Failed to get group member info: {}", e)))?;

        let dict = PyDict::default();
        dict.get_or_insert(vm, PyStr::from("card").into_pyobject(vm), || {
            PyStr::from(resp.card).into_pyobject(vm)
        })
        .ok();
        dict.get_or_insert(vm, PyStr::from("nickname").into_pyobject(vm), || {
            PyStr::from(resp.nickname).into_pyobject(vm)
        })
        .ok();
        return Ok(dict.into_pyref(vm));
    }
    #[pymethod]
    pub fn delete_msg(&self, mid: PyIntRef, vm: &VirtualMachine) -> PyResult<()> {
        self.client.delete_message_sync(bigint_to_i64(mid.as_bigint()))
            .map_err(|e| vm.new_value_error(format!("Failed to delete message: {}", e)))?;
        return Ok(());
    }
}
