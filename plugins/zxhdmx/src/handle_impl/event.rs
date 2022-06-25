use crate::{
    help_str::HELP_STR,
    utils::{check_player_in_game, transform_pyerr},
    InprType,
};
use anyhow::anyhow;
use countdown_bot3::countdown_bot::client::{CountdownBotClient, ResultType};
use pyvm::{
    builtins::{PyInt, PyStr},
    function::{FuncArgs, KwArgs, IntoPyObject},
    PyObjectRef, PyValue, PyRef, VirtualMachine,
};
use rustpython_vm as pyvm;
pub fn handle_event(
    user_id: i64,
    group_id: i64,
    inpr: InprType,
    game_object: PyObjectRef,
    message: String,
    client: CountdownBotClient,
) -> ResultType<()> {
    if !check_player_in_game(&inpr.lock().unwrap(), game_object.clone(), user_id)? {
        return Ok(());
    }
    let args = message.split(" ").collect::<Vec<&str>>();
    let pybool_true = inpr
        .lock()
        .unwrap()
        .enter(|vm| pyvm::eval::eval(vm, "True", vm.new_scope_with_builtins(), "<eval>"))
        .map_err(transform_pyerr)?;
    let pybool_false = inpr
        .lock()
        .unwrap()
        .enter(|vm| pyvm::eval::eval(vm, "False", vm.new_scope_with_builtins(), "<eval>"))
        .map_err(transform_pyerr)?;

    match args[0] {
        "帮助" => {
            client.send_group_msg_sync(group_id, HELP_STR, false)?;
        }
        "状态" => {
            client.send_group_msg_sync(
                group_id,
                inpr.lock()
                    .unwrap()
                    .enter(|vm| quick_call::<PyStr>(game_object.clone(), "get_status", vm, vec![]))?
                    .as_str(),
                false,
            )?;
        }
        "开始" => {
            inpr.lock()
                .unwrap()
                .enter(|vm| quick_call_no_ret(game_object.clone(), "start", vm, vec![]))?;
        }
        "拼点" => {
            inpr.lock().unwrap().enter(|vm| {
                quick_call_no_ret(
                    game_object.clone(),
                    "play",
                    vm,
                    vec![PyInt::from(user_id).into_pyobject(vm)],
                )
            })?;
        }
        cmd @ "选择" | cmd @ "更换题目" => {
            let problem_set = *args.get(1).ok_or(anyhow!("请输入要选择的问题集!"))?;
            inpr.lock().unwrap().enter(|vm| {
                quick_call_no_ret(
                    game_object.clone(),
                    "select",
                    vm,
                    vec![
                        PyInt::from(user_id).into_pyobject(vm),
                        PyStr::from(problem_set).into_pyobject(vm),
                        if cmd == "选择" {
                            pybool_false.clone()
                        } else {
                            pybool_true.clone()
                        },
                    ],
                )
            })?;
        }
        "查看物品" => {
            client.send_group_msg_sync(
                group_id,
                inpr.lock()
                    .unwrap()
                    .enter(|vm| {
                        quick_call::<PyStr>(
                            game_object.clone(),
                            "get_items",
                            vm,
                            vec![PyInt::from(user_id).into_pyobject(vm)],
                        )
                    })?
                    .as_str(),
                false,
            )?;
        }
        "使用物品" => {
            let item_id = *args.get(1).ok_or(anyhow!("请输入要使用的物品ID!"))?;
            let arg = *args.get(2).ok_or(anyhow!("请输入物品参数!"))?;
            inpr.lock().unwrap().enter(|vm| {
                quick_call_no_ret(
                    game_object.clone(),
                    "use_item",
                    vm,
                    vec![
                        PyInt::from(user_id).into_pyobject(vm),
                        PyStr::from(item_id).into_pyobject(vm),
                        PyStr::from(arg).into_pyobject(vm),
                    ],
                )
            })?;
        }
        "接受" => {
            inpr.lock().unwrap().enter(|vm| {
                quick_call_no_ret(
                    game_object.clone(),
                    "accept",
                    vm,
                    vec![PyInt::from(user_id).into_pyobject(vm)],
                )
            })?;
        }
        "提醒" => {
            inpr.lock().unwrap().enter(|vm| {
                quick_call_no_ret(game_object.clone(), "notify_non_played", vm, vec![])
            })?;
        }
        "跳过" => {
            inpr.lock().unwrap().enter(|vm| {
                quick_call_no_ret(game_object.clone(), "skip_non_played", vm, vec![])
            })?;
        }
        "终止" => {
            inpr.lock()
                .unwrap()
                .enter(|vm| quick_call_no_ret(game_object.clone(), "force_stop", vm, vec![]))?;
        }
        _ => return Ok(()),
    };
    return Ok(());
}

fn quick_call<T>(
    obj: PyObjectRef,
    func_name: &str,
    vm: &VirtualMachine,
    args: Vec<PyObjectRef>,
) -> anyhow::Result<PyRef<T>>
where
    T: PyValue,
{
    return vm
        .call_method(&obj, func_name, FuncArgs::new(args, KwArgs::default()))
        .map_err(transform_pyerr)?
        .downcast::<T>()
        .map_err(|_| anyhow!("Failed to perform type cast!"));
}
fn quick_call_no_ret(
    obj: PyObjectRef,
    func_name: &str,
    vm: &VirtualMachine,
    args: Vec<PyObjectRef>,
) -> anyhow::Result<()> {
    return vm
        .call_method(&obj, func_name, FuncArgs::new(args, KwArgs::default()))
        .map_err(transform_pyerr)
        .map(|_| ());
}
