use crate::{
    config::ZxhdmxConfig,
    pytypes::{
        wrapped_bot::{MyPyLogger, WrappedCountdownBot},
        wrapped_plugin::{WrappedConfig, WrappedPlugin},
    },
    utils::{check_player_in_game, transform_pyerr},
    DataType, GameObjectType, InprType,
};
use countdown_bot3::countdown_bot::client::{CountdownBotClient, ResultType};
use log::debug;
use pyvm::{
    builtins::{PyInt, PyModule},
    function::{FuncArgs, KwArgs, PosArgs, IntoPyObject},
    PyValue, PyRef,
};
use rustpython_vm as pyvm;
pub fn handle_command(
    user_id: i64,
    group_id: i64,
    client: CountdownBotClient,
    game_data: DataType,
    game_module: PyRef<PyModule>,
    game_objects: GameObjectType,
    inpr: InprType,
    config: ZxhdmxConfig,
) -> ResultType<()> {
    debug!("Entered blocking task..");
    let mut game_objects = game_objects.lock().unwrap();
    if !game_objects.contains_key(&group_id) {
        debug!("Locked! Calling python..");
        // let game_class =
        let game_obj = inpr
            .lock()
            .unwrap()
            .enter(|vm| {
                let game_class = game_module.get_attr("Game", vm)?;
                // vm.call_method(obj, method_name, args)
                // let call_method =
                //     PyMethod::get(game_class, PyStr::from("__call__").into_ref(vm), vm)?;
                let obj = vm.call_method(
                    &game_class,
                    "__call__",
                    FuncArgs::new(
                        PosArgs::new(vec![
                            WrappedCountdownBot::new((MyPyLogger {}).into_ref(vm), client.clone())
                                .into_pyobject(vm),
                            PyInt::from(group_id).into_pyobject(vm),
                            WrappedPlugin::new(
                                game_data.clone(),
                                WrappedConfig::new(config.clone()).into_ref(vm),
                            )
                            .into_pyobject(vm),
                        ]),
                        KwArgs::default(),
                    ),
                )?;
                return Ok(obj);
            })
            .map_err(transform_pyerr)?;
        game_objects.insert(group_id, game_obj);
    }
    let game_inst = game_objects.get(&group_id).unwrap().clone();
    let py_inpr = inpr.lock().unwrap();
    let method_name = if check_player_in_game(&py_inpr, game_inst.clone(), user_id)? {
        "exit"
    } else {
        "join"
    };
    py_inpr
        .enter(|vm| {
            vm.call_method(
                &game_inst,
                method_name,
                FuncArgs::new(
                    PosArgs::new(vec![PyInt::from(user_id).into_pyobject(vm)]),
                    KwArgs::default(),
                ),
            )
            // PyMethod::get(game_inst, PyStr::from(method_name).into_ref(vm), vm)?.invoke(
            //    ,
            //     vm,
            // )
        })
        .map_err(transform_pyerr)?;
    return Ok(());
}
