use std::collections::{HashMap, HashSet};
use std::string::ToString;

use v_common::module::remote_indv_r_storage::get_individual;
use v_common::module::veda_backend::indv_apply_cmd;
use v_common::onto::individual::Individual;
use v_common::onto::onto_impl::Onto;
use v_common::onto::parser::parse_raw;
use v_common::v_api::api_client::UpdateOptions;
use v_common::v_api::api_client::{IndvOp, MStorageClient, ALL_MODULES};
use v_common::v_api::obj::ResultCode;

#[derive(Default)]
pub struct CallbackSharedData {
    pub g_key2indv: HashMap<String, Individual>,
    pub g_key2attr: HashMap<String, String>,
}

impl CallbackSharedData {
    pub fn set_g_parent_script_id_etc(&mut self, event_id: &str) {
        let mut event_id = event_id;

        if !event_id.is_empty() {
            let mut aa: Vec<&str> = event_id.split(';').collect();
            if !aa.is_empty() {
                event_id = aa.first().unwrap();
            }

            aa = event_id.split('+').collect();

            if aa.len() >= 2 {
                self.g_key2attr.insert("$parent_script_id".to_owned(), aa.get(1).unwrap().to_string());
                self.g_key2attr.insert("$parent_document_id".to_owned(), aa.first().unwrap().to_string());
            } else {
                self.g_key2attr.insert("$parent_script_id".to_owned(), String::default());
                self.g_key2attr.insert("$parent_document_id".to_owned(), String::default());
            }
        } else {
            self.g_key2attr.insert("$parent_script_id".to_owned(), String::default());
            self.g_key2attr.insert("$parent_document_id".to_owned(), String::default());
        }
    }

    pub fn set_g_super_classes(&mut self, indv_types: &[String], onto: &Onto) {
        let mut super_classes = HashSet::new();

        for indv_type in indv_types.iter() {
            onto.get_supers(indv_type, &mut super_classes);
        }

        let mut g_super_classes = String::new();

        g_super_classes.push('[');
        for s in super_classes.iter() {
            if g_super_classes.len() > 2 {
                g_super_classes.push(',');
            }
            g_super_classes.push('"');
            g_super_classes.push_str(s);
            g_super_classes.push('"');
        }
        g_super_classes.push(']');

        self.g_key2attr.insert("$super_classes".to_owned(), g_super_classes);
    }
}

pub struct TransactionItem {
    uri: String,
    pub cmd: IndvOp,
    pub indv: Individual,
    ticket_id: String,
    pub rc: ResultCode,
}

pub struct Transaction {
    pub sys_ticket: String,
    pub id: i64,
    pub event_id: String,
    buff: HashMap<String, usize>,
    pub queue: Vec<TransactionItem>,
    pub src: String,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            sys_ticket: "".to_string(),
            id: 0,
            event_id: "".to_string(),
            buff: Default::default(),
            queue: vec![],
            src: "".to_string(),
        }
    }
}

impl Transaction {
    fn add_item(&mut self, item: TransactionItem) {
        self.buff.insert(item.uri.clone(), self.queue.len());
        self.queue.push(item);
    }

    pub(crate) fn get_indv(&mut self, id: &str) -> Option<&mut Individual> {
        if let Some(idx) = self.buff.get(id) {
            if let Some(ti) = self.queue.get_mut(*idx) {
                return Some(&mut ti.indv);
            }
        }

        None
    }

    pub(crate) fn add_to_transaction(&mut self, cmd: IndvOp, new_indv: Individual, ticket_id: String, _user_id: String) -> ResultCode {
        let mut ti = TransactionItem {
            uri: "".to_string(),
            cmd,
            indv: new_indv,
            ticket_id,
            rc: ResultCode::Ok,
        };

        if ti.cmd == IndvOp::Remove {
            // No changes needed for Remove operation
        } else {
            ti.uri = ti.indv.get_id().to_string();

            if ti.cmd == IndvOp::AddTo || ti.cmd == IndvOp::SetIn || ti.cmd == IndvOp::RemoveFrom {
                if let Some(prev_indv) = self.get_indv(ti.indv.get_id()) {
                    debug!("{:?} BEFORE: {}", ti.cmd, &prev_indv);
                    debug!("{:?} APPLY: {}", ti.cmd, &ti.indv);
                    indv_apply_cmd(&ti.cmd, prev_indv, &mut ti.indv);
                    debug!("{:?} AFTER: {}", ti.cmd, &prev_indv);
                    ti.indv = Individual::new_from_obj(prev_indv.get_obj());
                } else {
                    match get_individual(ti.indv.get_id()) {
                        Ok(Some(mut prev_indv)) => {
                            if parse_raw(&mut prev_indv).is_ok() {
                                prev_indv.parse_all();
                                debug!("{:?} BEFORE: {}", ti.cmd, &prev_indv);
                                debug!("{:?} APPLY: {}", ti.cmd, &ti.indv);
                                indv_apply_cmd(&ti.cmd, &mut prev_indv, &mut ti.indv);
                                debug!("{:?} AFTER: {}", ti.cmd, &prev_indv);
                                ti.indv = prev_indv;
                            } else {
                                ti.rc = ResultCode::UnprocessableEntity;
                            }
                        },
                        Ok(None) => {
                            // Individual not found
                            ti.rc = ResultCode::NotFound;
                        },
                        Err(e) => {
                            // Error occurred while getting individual
                            error!("Error getting individual: {:?}", e);
                            ti.rc = e;
                        },
                    }
                }

                if ti.rc == ResultCode::Ok {
                    ti.cmd = IndvOp::Put;
                }
            }
        }

        if ti.rc == ResultCode::Ok {
            self.add_item(ti);
            ResultCode::Ok
        } else {
            ti.rc
        }
    }
}

pub fn commit(tnx: &Transaction, api_client: &mut MStorageClient) -> ResultCode {
    for ti in tnx.queue.iter() {
        if ti.cmd == IndvOp::Remove && ti.indv.get_id().is_empty() {
            continue;
        }

        if ti.rc != ResultCode::Ok {
            return ti.rc;
        }

        if ti.indv.get_id().is_empty() || ti.indv.get_id().len() < 2 {
            warn!("skip individual with invalid id: {}", ti.indv.to_string());
            continue;
        }

        debug!("commit {}", &ti.indv);
        let update_options = UpdateOptions {
            event_id: Some(&tnx.event_id),
            src: Some(&tnx.src),
            assigned_subsystems: Some(ALL_MODULES),
            addr: None,
        };

        match api_client.update(&ti.indv, &ti.ticket_id, ti.cmd.clone(), update_options) {
            Ok(res) => {
                if res.result != ResultCode::Ok {
                    error!("commit: op_id={}, code={:?}", res.op_id, res.result);
                    return res.result;
                }
            },
            Err(e) => {
                return e.result;
            },
        }
    }

    ResultCode::Ok
}
