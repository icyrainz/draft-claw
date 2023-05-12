use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::{env, result};

use lazy_static::lazy_static;
use strum_macros::EnumString;

pub type Res<T> = Result<T, String>;

pub trait ErrToStr<T, E: Display> {
    fn err_to_str(self) -> Res<T>;
}

impl<T, E: Display> ErrToStr<T, E> for result::Result<T, E> {
    fn err_to_str(self) -> Res<T> {
        self.map_err(|err| err.to_string())
    }
}

#[derive(strum_macros::Display, Eq, PartialEq, Debug, Hash)]
pub enum DbgFlg {
    #[strum(serialize = "DBG_FLG_DB")]
    Db,
    #[strum(serialize = "DBG_FLG_BOT")]
    Bot,
}

lazy_static! {
    pub static ref DBG_FLG_DEFAULTS: HashMap<DbgFlg, bool> =
        HashMap::from([(DbgFlg::Db, true), (DbgFlg::Bot, true)]);
}

pub trait DebugIf: Debug {
    fn dbg(&self);
    fn dbg_if(&self, flg: DbgFlg);
}

impl<T: Debug> DebugIf for T {
    fn dbg(&self)
    where
        Self: Debug,
    {
        dbg!(self);
    }

    fn dbg_if(&self, flg: DbgFlg)
    where
        Self: Debug,
    {
        if checkflag(&flg) {
            dbg!(self);
        }
    }
}

pub fn log_if(s: &str, flg: DbgFlg) {
    if checkflag(&flg) {
        println!("{} {}", utc_now(), s);
    }
}

fn utc_now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
fn checkflag(flag: &DbgFlg) -> bool {
    let flags_to_check = vec![DbgFlg::Db];
    flags_to_check.iter().any(|flg| {
        env::var(flg.to_string()).ok().map_or_else(
            || DBG_FLG_DEFAULTS.get(flg).unwrap_or(&false).to_owned(),
            |s| s == "1" || s == "true",
        )
    })
}
