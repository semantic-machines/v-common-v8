use crate::common::*;
use crate::session_cache::*;
use std::cell::RefCell;
use std::sync::Mutex;
use v8::{Context, GetPropertyNamesArgs, HandleScope, Local};
use v_common::az_impl::az_lmdb::LmdbAzContext;
use v_common::module::module_impl::Module;
use v_common::module::remote_indv_r_storage::get_individual;
use v_common::onto::individual::Individual;
use v_common::onto::parser::parse_raw;
use v_common::search::common::FTQuery;
use v_common::search::ft_client::*;
use v_common::v_api::api_client::IndvOp;
use v_common::v_authorization::common::{Access, AuthorizationContext, ACCESS_8_LIST, ACCESS_PREDICATE_LIST};

lazy_static! {
    static ref AZ: Mutex<RefCell<LmdbAzContext>> = Mutex::new(RefCell::new(LmdbAzContext::new(1000)));
    static ref FT_CLIENT: Mutex<RefCell<FTClient>> = Mutex::new(RefCell::new(FTClient::new(Module::get_property("ft_query_service_url").unwrap_or_default())));
    pub static ref G_VARS: Mutex<RefCell<CallbackSharedData>> = Mutex::new(RefCell::new(CallbackSharedData::default()));
    pub static ref G_TRANSACTION: Mutex<RefCell<Transaction>> = Mutex::new(RefCell::new(Transaction::default()));
}

pub fn fn_callback_get_rights(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    //let ticket = get_string_arg(scope, &args, 0, "callback_get_rights: ticket not found or invalid").unwrap_or_default();
    let id = get_string_arg(scope, &args, 1, None).unwrap_or_default();
    let user_id = if let Some(u) = get_string_arg(scope, &args, 2, None) {
        u
    } else {
        let key = v8::String::new(scope, "user_uri").unwrap();
        let user_uri = scope.get_current_context().global(scope).get(scope, key.into()).unwrap().to_string(scope).unwrap();
        user_uri.to_rust_string_lossy(scope)
    };

    debug!("user_id={}, doc_id={}", user_id, id);

    let mut sh_az = AZ.lock().unwrap();
    let az = sh_az.get_mut();

    let rights = az.authorize(&id, &user_id, Access::CanRead as u8 | Access::CanCreate as u8 | Access::CanDelete as u8 | Access::CanUpdate as u8, false).unwrap_or(0);

    let mut pstm = Individual::default();
    pstm.set_id("_");
    pstm.add_uri("rdf:type", "v-s:PermissionStatement");
    for ch_access in ACCESS_8_LIST {
        if rights & ch_access > 0 {
            pstm.add_bool(ACCESS_PREDICATE_LIST[ch_access as usize], rights & ch_access > 0);
        }
    }
    let j_indv = individual2v8obj(scope, pstm.parse_all());
    rv.set(j_indv.into());

    drop(sh_az);
}

pub fn fn_callback_get_individual(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    let id = get_string_arg(scope, &args, 1, Some("callback_get_individual: id not found or invalid")).unwrap_or_default();

    if id == "undefined" {
        // Do nothing
    } else if id.starts_with('$') {
        let mut sh_g_vars = G_VARS.lock().unwrap();
        let g_vars = sh_g_vars.get_mut();

        if let Some(indv) = g_vars.g_key2indv.get_mut(&id) {
            let j_indv = individual2v8obj(scope, indv.parse_all());
            rv.set(j_indv.into());
        }
        drop(sh_g_vars);
    } else {
        let mut sh_tnx = G_TRANSACTION.lock().unwrap();
        let tnx = sh_tnx.get_mut();

        if let Some(indv) = tnx.get_indv(&id) {
            let j_indv = individual2v8obj(scope, indv);
            rv.set(j_indv.into());
        } else {
            match get_individual(&id) {
                Ok(Some(mut indv)) => {
                    if parse_raw(&mut indv).is_ok() {
                        let j_indv = individual2v8obj(scope, indv.parse_all());
                        rv.set(j_indv.into());
                    } else {
                        let error_msg = format!("Failed to parse binobj for id: {}", id);
                        error!("callback_get_individual: {}", error_msg);
                        let error_string = v8::String::new(scope, &error_msg).unwrap();
                        let error = v8::Exception::error(scope, error_string);
                        scope.throw_exception(error);
                    }
                },
                Ok(None) => {
                    let warn_msg = format!("Individual not found for id: {}", id);
                    warn!("callback_get_individual: {}", warn_msg);
                    rv.set(v8::undefined(scope).into());
                },
                Err(e) => {
                    let error_msg = format!("Error getting individual for id: {}, error: {:?}", id, e);
                    error!("callback_get_individual: {}", error_msg);
                    let error_string = v8::String::new(scope, &error_msg).unwrap();
                    let error = v8::Exception::error(scope, error_string);
                    scope.throw_exception(error);
                },
            }
        }

        drop(sh_tnx);
    }
}

pub fn fn_callback_get_individuals(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    let j_res = v8::Array::new(scope, 0);

    let arg1 = args.get(1);
    if arg1.is_array() {
        if let Some(r) = arg1.to_object(scope) {
            if let Some(arr_keys) = r.get_property_names(scope, GetPropertyNamesArgs::default()) {
                let mut sh_tnx = G_TRANSACTION.lock().unwrap();
                let tnx = sh_tnx.get_mut();

                for idx in 0..arr_keys.length() {
                    let j_idx = v8::Integer::new(scope, idx as i32);
                    let j_id = r.get(scope, j_idx.into()).unwrap().to_object(scope).unwrap();
                    let id = j_id.to_string(scope).unwrap().to_rust_string_lossy(scope);

                    if let Some(indv) = tnx.get_indv(&id) {
                        let j_indv = individual2v8obj(scope, indv);
                        j_res.set(scope, j_idx.into(), j_indv.into());
                    } else {
                        match get_individual(&id) {
                            Ok(Some(mut indv)) => {
                                if parse_raw(&mut indv).is_ok() {
                                    let j_indv = individual2v8obj(scope, indv.parse_all());
                                    j_res.set(scope, j_idx.into(), j_indv.into());
                                } else {
                                    let error_msg = format!("Failed to parse binobj for id: {}", id);
                                    error!("callback_get_individuals: {}", error_msg);
                                    let error_string = v8::String::new(scope, &error_msg).unwrap();
                                    let error = v8::Exception::error(scope, error_string);
                                    scope.throw_exception(error);
                                    return;
                                }
                            },
                            Ok(None) => {
                                warn!("callback_get_individuals: individual not found, id={}", id);
                                let null_value = v8::null(scope);
                                j_res.set(scope, j_idx.into(), null_value.into());
                            },
                            Err(e) => {
                                let error_msg = format!("Error getting individual, id={}, error={:?}", id, e);
                                error!("callback_get_individuals: {}", error_msg);
                                let error_string = v8::String::new(scope, &error_msg).unwrap();
                                let error = v8::Exception::error(scope, error_string);
                                scope.throw_exception(error);
                                return;
                            },
                        }
                    }
                }
                drop(sh_tnx);
            }
        }
    } else {
        let error_msg = "Argument is not an array";
        error!("callback_get_individuals: {}", error_msg);
        let error_string = v8::String::new(scope, error_msg).unwrap();
        let error = v8::Exception::type_error(scope, error_string);
        scope.throw_exception(error);
        return;
    }
    rv.set(j_res.into());
}

pub fn fn_callback_print(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _rv: v8::ReturnValue) {
    let mut str_out = String::new();

    for idx in 0..args.length() {
        let arg = args.get(idx);
        str_out.push_str(&arg.to_string(scope).unwrap().to_rust_string_lossy(scope));
        str_out.push(' ');
    }
    info!("{}", str_out);
}

pub fn fn_callback_log_trace(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _rv: v8::ReturnValue) {
    let arg1 = args.get(0);
    info!("{}", arg1.to_string(scope).unwrap().to_rust_string_lossy(scope));
}

pub fn fn_callback_get_env_str_var(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    if let Some(var_name) = get_string_arg(scope, &args, 0, Some("fn_callback_get_env_str_var: arg not found or invalid")) {
        let mut sh_g_vars = G_VARS.lock().unwrap();
        let g_vars = sh_g_vars.get_mut();

        debug!("fn_callback_get_env_str_var, var_name={:?}", var_name);

        if let Some(v) = g_vars.g_key2attr.get(&var_name) {
            let j_res = str_2_v8(scope, v);
            rv.set(j_res.into());
        }

        drop(sh_g_vars);
    }
}
pub fn fn_callback_get_env_num_var(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut _rv: v8::ReturnValue) {
    if let Some(var_name) = get_string_arg(scope, &args, 0, Some("fn_callback_get_env_str_var: arg not found or invalid")) {
        //let mut sh_g_vars = G_VARS.lock().unwrap();
        //let _g_vars = sh_g_vars.get_mut();

        debug!("fn_callback_get_env_num_var, var_name={:?}", var_name);

        //if var_name == "$queue_elements_count" || var_name == "$queue_elements_processed" {
        //    return;
        //}
    }
}

pub fn fn_callback_query(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    let ticket = get_string_arg(scope, &args, 0, Some("callback_query: arg0 [ticket] not found or invalid"));
    if ticket.is_none() {
        return;
    }
    let mut ticket = ticket.unwrap();

    let query = get_string_arg(scope, &args, 1, Some("callback_query: arg1 [query] not found or invalid"));
    if query.is_none() {
        return;
    }

    let sort;
    let databases;
    let top;
    let limit;
    let from;

    if ticket.is_empty() {
        let mut sh_tnx = G_TRANSACTION.lock().unwrap();
        let tnx = sh_tnx.get_mut();
        ticket = tnx.sys_ticket.to_owned();
        drop(sh_tnx);
    }

    let mut query = FTQuery::new_with_ticket(&ticket, &query.unwrap());
    if args.length() > 2 {
        sort = get_string_arg(scope, &args, 2, Some("callback_query: arg2 [sort] not found or invalid"));
        query.sort = sort.unwrap_or_default();
        if args.length() > 3 {
            databases = get_string_arg(scope, &args, 3, Some("callback_query: arg3 [databases] not found or invalid"));
            query.databases = databases.unwrap_or_default();
            if args.length() > 4 {
                top = get_string_i32(scope, &args, 4, Some("callback_query: arg4 [top] not found or invalid"));
                query.top = top.unwrap_or(100000);
                if args.length() > 5 {
                    limit = get_string_i32(scope, &args, 5, Some("callback_query: arg5 [limit] not found or invalid"));
                    query.limit = limit.unwrap_or(100000);
                    if args.length() > 6 {
                        from = get_string_i32(scope, &args, 6, Some("callback_query: arg6 [from] not found or invalid"));
                        query.from = from.unwrap_or_default();
                    }
                }
            }
        }
    }

    let _context = scope.get_current_context();

    let mut sh_ft_client = FT_CLIENT.lock().unwrap();
    let ft_client = sh_ft_client.get_mut();

    let res = ft_client.query(query);

    drop(sh_ft_client);

    let j_res = query_result2v8obj(scope, &res);
    rv.set(j_res.into());
}

fn fn_callback_update(opt: IndvOp, scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    let wticket = get_string_arg(scope, &args, 0, Some("fn_callback_update: arg0 [ticket] not found or invalid"));
    if wticket.is_none() {
        return;
    }
    let mut ticket = wticket.unwrap_or_default();

    let arg1 = args.get(1);
    let mut indv;

    if opt == IndvOp::Remove {
        indv = Individual::default();

        if arg1.is_string() {
            if let Some(id) = arg1.to_string(scope) {
                indv.set_id(&id.to_rust_string_lossy(scope));
            }
        } else {
            error!("callback {:?}, argument is not string", opt);
        }
    } else if arg1.is_object() {
        let js_obj = arg1.to_object(scope).unwrap();
        indv = v8obj2individual(scope, js_obj);
    } else {
        indv = Individual::default();
        error!("callback {:?}, argument is not object", opt);
    }

    if !indv.get_id().is_empty() {
        let mut sh_tnx = G_TRANSACTION.lock().unwrap();
        let tnx = sh_tnx.get_mut();

        if ticket.is_empty() {
            ticket = tnx.sys_ticket.to_owned();
        }

        debug!("ADD TO TRANSACTION {:?} {}", &opt, indv.get_id());
        let res = tnx.add_to_transaction(opt, indv, ticket, "".to_string());
        debug!("res={:?}", res);

        rv.set(v8::Integer::new(scope, res as i32).into());
    } else {
        error!("callback {:?}, invalid argument", opt);
    }
}

pub fn fn_callback_put_individual(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue) {
    fn_callback_update(IndvOp::Put, scope, args, rv);
}

pub fn fn_callback_remove_individual(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue) {
    fn_callback_update(IndvOp::Remove, scope, args, rv);
}
pub fn fn_callback_add_to_individual(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue) {
    fn_callback_update(IndvOp::AddTo, scope, args, rv);
}
pub fn fn_callback_set_in_individual(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue) {
    fn_callback_update(IndvOp::SetIn, scope, args, rv);
}
pub fn fn_callback_remove_from_individual(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue) {
    fn_callback_update(IndvOp::RemoveFrom, scope, args, rv);
}

pub fn init_context_with_callback<'a>(scope: &mut HandleScope<'a, ()>) -> Local<'a, Context> {
    let object_templ = v8::ObjectTemplate::new(scope);
    object_templ.set(str_2_v8(scope, "print").into(), v8::FunctionTemplate::new(scope, fn_callback_print).into());
    object_templ.set(str_2_v8(scope, "get_individual").into(), v8::FunctionTemplate::new(scope, fn_callback_get_individual).into());
    object_templ.set(str_2_v8(scope, "get_individuals").into(), v8::FunctionTemplate::new(scope, fn_callback_get_individuals).into());
    object_templ.set(str_2_v8(scope, "put_individual").into(), v8::FunctionTemplate::new(scope, fn_callback_put_individual).into());
    object_templ.set(str_2_v8(scope, "get_env_str_var").into(), v8::FunctionTemplate::new(scope, fn_callback_get_env_str_var).into());
    object_templ.set(str_2_v8(scope, "get_env_num_var").into(), v8::FunctionTemplate::new(scope, fn_callback_get_env_num_var).into());
    object_templ.set(str_2_v8(scope, "query").into(), v8::FunctionTemplate::new(scope, fn_callback_query).into());
    object_templ.set(str_2_v8(scope, "remove_individual").into(), v8::FunctionTemplate::new(scope, fn_callback_remove_individual).into());
    object_templ.set(str_2_v8(scope, "add_to_individual").into(), v8::FunctionTemplate::new(scope, fn_callback_add_to_individual).into());
    object_templ.set(str_2_v8(scope, "set_in_individual").into(), v8::FunctionTemplate::new(scope, fn_callback_set_in_individual).into());
    object_templ.set(str_2_v8(scope, "remove_from_individual").into(), v8::FunctionTemplate::new(scope, fn_callback_remove_from_individual).into());
    object_templ.set(str_2_v8(scope, "log_trace").into(), v8::FunctionTemplate::new(scope, fn_callback_log_trace).into());
    object_templ.set(str_2_v8(scope, "get_rights").into(), v8::FunctionTemplate::new(scope, fn_callback_get_rights).into());

    v8::Context::new_from_template(scope, object_templ)
}

fn get_string_arg(scope: &mut v8::HandleScope, args: &v8::FunctionCallbackArguments, idx: i32, warn_msg: Option<&str>) -> Option<String> {
    let arg = args.get(idx);

    if !arg.is_null_or_undefined() {
        if let Some(arg) = arg.to_string(scope) {
            return Some(arg.to_rust_string_lossy(scope));
        }
    }

    if let Some(msg) = warn_msg {
        warn!("{}", msg);
    }
    None
}

fn get_string_i32(scope: &mut v8::HandleScope, args: &v8::FunctionCallbackArguments, idx: i32, warn_msg: Option<&str>) -> Option<i32> {
    let arg = args.get(idx).int32_value(scope);
    if arg.is_none() {
        if let Some(msg) = warn_msg {
            warn!("{}", msg);
        }
        return None;
    }
    arg
}
